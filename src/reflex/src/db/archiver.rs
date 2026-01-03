use std::sync::Arc;
use object_store::{ObjectStore, path::Path};
use object_store::aws::AmazonS3Builder;
use tokio_postgres::Client;
use parquet::arrow::ArrowWriter;
use arrow::datatypes::{Schema, Field, DataType, TimeUnit};
use arrow::array::{Float64Array, TimestampNanosecondArray, RecordBatch};

pub struct Archiver {
    s3: Arc<dyn ObjectStore>,
    pg: Arc<Client>,
    bucket: String,
}

impl Archiver {
    pub async fn new(pg_client: Client) -> Result<Self, Box<dyn std::error::Error>> {
        // Load R2 Config from Env
        let access_key = std::env::var("CLOUDFLARE_ACCESS_KEY_ID")
            .expect("❌ Missing CLOUDFLARE_ACCESS_KEY_ID");
        
        let secret_key = std::env::var("CLOUDFLARE_SECRET_ACCESS_KEY")
            .or_else(|_| std::env::var("CLOUDFLARE_SECRET_ACCESS_KEY_ID"))
            .expect("❌ Missing CLOUDFLARE_SECRET_ACCESS_KEY or _ID");

        let bucket_name = std::env::var("CLOUDFLARE_BUCKET_NAME").unwrap_or("voltaire".to_string());
        
        let endpoint = std::env::var("CLOUDFLARE_STORAGE_URL")
            .or_else(|_| {
                 let account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID")?;
                 Ok::<String, std::env::VarError>(format!("https://{}.r2.cloudflarestorage.com", account_id))
            })
            .expect("❌ Missing CLOUDFLARE_STORAGE_URL or ACCOUNT_ID");

        let s3 = AmazonS3Builder::new()
            .with_endpoint(&endpoint)
            .with_access_key_id(&access_key)
            .with_secret_access_key(&secret_key)
            .with_region("auto") // Required for R2
            .with_bucket_name(&bucket_name)
            .build()?;

        Ok(Self {
            s3: Arc::new(s3),
            pg: Arc::new(pg_client),
            bucket: bucket_name,
        })
    }

    /// Finds partitions in ohlcv_1min that are older than `retention_days`.
    /// Returns a list of partition names (e.g., '2023-01').
    pub async fn find_cold_partitions(&self, table: &str, retention_days: i64) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        println!("🔍 Scanning for Cold Partitions in '{}' (> {} days)...", table, retention_days);
        
        let query = format!(
            "SELECT name, maxTimestamp FROM table_partitions('{}') 
             WHERE maxTimestamp < dateadd('d', -{}, now())
             ORDER BY maxTimestamp ASC", 
            table, retention_days
        );

        let rows = self.pg.query(&query, &[]).await?;
        let mut partitions = Vec::new();

        for row in rows {
            let name: String = row.get("name");
            partitions.push(name);
        }
        
        Ok(partitions)
    }

    /// Exports a specific partition to Parquet on R2
    pub async fn archive_partition(&self, table: &str, partition: &str, time_col: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("📦 Archiving Partition: {}/{}", table, partition);
        
        // 1. Define Arrow Schema for OHLCV
        let schema = Arc::new(Schema::new(vec![
            Field::new("ts", DataType::Timestamp(TimeUnit::Nanosecond, None), false),
            Field::new("open", DataType::Float64, false),
            Field::new("high", DataType::Float64, false),
            Field::new("low", DataType::Float64, false),
            Field::new("close", DataType::Float64, false),
            Field::new("volume", DataType::Float64, false),
        ]));

        // 2. Fetch Data from QuestDB
        let range_query = format!("SELECT minTimestamp, maxTimestamp FROM table_partitions('{}') WHERE name = '{}'", table, partition);
        let range_row = self.pg.query_one(&range_query, &[]).await?;
        
        let min_ts: std::time::SystemTime = range_row.get(0);
        let max_ts: std::time::SystemTime = range_row.get(1);
        
        // Prepare Data Stream
        // Use SQL Aliasing to normalize timestamp column name and cast volume to double
        let data_query = format!(
            "SELECT {} as ts, open, high, low, close, cast(volume as double) FROM \"{}\" WHERE {} BETWEEN $1 AND $2 ORDER BY {} ASC",
            time_col, table, time_col, time_col
        );
        
        let stmt = self.pg.prepare(&data_query).await?;
        let rows = self.pg.query(&stmt, &[&min_ts, &max_ts]).await?;
        
        if rows.is_empty() {
            println!("⚠️ Partition {} is empty. Skipping.", partition);
            return Ok(());
        }
        
        println!("   Fetched {} rows. Converting to Arrow...", rows.len());

        // 3. Convert Rows to Arrow Columns
        let mut ts_builder = Vec::with_capacity(rows.len());
        let mut open_builder = Vec::with_capacity(rows.len());
        let mut high_builder = Vec::with_capacity(rows.len());
        let mut low_builder = Vec::with_capacity(rows.len());
        let mut close_builder = Vec::with_capacity(rows.len());
        let mut vol_builder = Vec::with_capacity(rows.len());

        for row in rows {
            let ts: std::time::SystemTime = row.get(0); // Now always index 0 because of alias
            let ts_nanos = ts.duration_since(std::time::UNIX_EPOCH)?.as_nanos() as i64;
            
            ts_builder.push(ts_nanos);
            open_builder.push(row.get::<_, f64>(1));
            high_builder.push(row.get::<_, f64>(2));
            low_builder.push(row.get::<_, f64>(3));
            close_builder.push(row.get::<_, f64>(4));
            vol_builder.push(row.get::<_, f64>(5));
        }

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(TimestampNanosecondArray::from(ts_builder)),
                Arc::new(Float64Array::from(open_builder)),
                Arc::new(Float64Array::from(high_builder)),
                Arc::new(Float64Array::from(low_builder)),
                Arc::new(Float64Array::from(close_builder)),
                Arc::new(Float64Array::from(vol_builder)),
            ],
        )?;

        // 4. Buffer Parquet in Memory (Synchronous)
        let mut buffer = Vec::new();
        let props = parquet::file::properties::WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(&mut buffer, schema.clone(), Some(props))?;
        writer.write(&batch)?;
        writer.close()?;
        
        let buffer_size = buffer.len();
        println!("   Parquet Size: {:.2} KB", buffer_size as f64 / 1024.0);

        // 5. Upload to R2 (Single Put)
        let file_path = Path::from(format!("archives/{}/{}.parquet", table, partition));
        println!("   Uploading to R2: s3://{}/{}", self.bucket, file_path);
        
        self.s3.put(&file_path, buffer.into()).await?;

        println!("✅ Archive Successful: {}", file_path);
        
        // 6. Atomic Drop
        self.drop_partition(table, partition).await?;

        Ok(())
    }

    async fn drop_partition(&self, table: &str, partition: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🗑️ Dropping Local Partition: {}...", partition);
        // QuestDB: ALTER TABLE table_name DROP PARTITION 'partition_name' -- Table name identifier
        let drop_query = format!("ALTER TABLE {} DROP PARTITION '{}'", table, partition); 
        self.pg.simple_query(&drop_query).await?;
        println!("✅ Local Partition Dropped.");
        Ok(())
    }
}

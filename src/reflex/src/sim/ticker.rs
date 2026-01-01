use tokio_postgres::{Client, NoTls};
use futures_util::{StreamExt, TryStreamExt};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::market::Tick;
use std::sync::Arc;
use object_store::{ObjectStore, path::Path};
use object_store::aws::AmazonS3Builder;
use parquet::arrow::async_reader::ParquetRecordBatchStreamBuilder;
use futures::stream::{self, Stream};
use std::pin::Pin;
use chrono::{DateTime, Utc, Datelike, Duration};
use std::env;

pub struct SimTicker {
    client: Client,
    r2_store: Option<Arc<dyn ObjectStore>>,
}

impl SimTicker {
    pub async fn new(db_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (client, connection) = tokio_postgres::connect(db_url, NoTls).await?;

        // Spawn output connection task
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("❌ SimTicker Connection Error: {}", e);
            }
        });

        // Initialize R2 Store if credentials exist
        let r2_store = if let (Ok(key), Ok(secret), Ok(bucket), Ok(endpoint)) = (
            env::var("CLOUDFLARE_ACCESS_KEY_ID"),
            env::var("CLOUDFLARE_SECRET_ACCESS_KEY_ID"),
            env::var("CLOUDFLARE_BUCKET_NAME"),
            env::var("CLOUDFLARE_STORAGE_URL"),
        ) {
            println!("☁️  SimTicker: R2 Credentials Found. Initializing Object Store...");
            let s3 = AmazonS3Builder::new()
                .with_region("auto")
                .with_endpoint(endpoint)
                .with_access_key_id(key)
                .with_secret_access_key(secret)
                .with_bucket_name(bucket)
                .build()?;
            Some(Arc::new(s3) as Arc<dyn ObjectStore>)
        } else {
            println!("⚠️  SimTicker: R2 Credentials Missing. Cloud failover disabled.");
            None
        };

        Ok(Self { client, r2_store })
    }

    /// Stream historical trades.
    /// L1: Check QuestDB.
    /// L3: If missing, failover to R2 Parquet Stream.
    pub async fn stream_history(
        self,
        symbol: &str,
        start_ts: i64, 
        end_ts: i64
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Tick, Box<dyn std::error::Error + Send + Sync + 'static>>> + Send>>, Box<dyn std::error::Error>> {
        
        // 1. L1 Check: Do we have data?
        // We'll peek at the count. A count query is usually fast in QuestDB.
        let check_query = format!(
            "SELECT count() FROM ohlcv_1min WHERE symbol = '{}' AND ts BETWEEN '{}' AND '{}'",
            symbol,
            to_quest_timestamp(start_ts),
            to_quest_timestamp(end_ts)
        );

        let row = self.client.query_one(&check_query, &[]).await?;
        let count: i64 = row.get(0);

        if count > 0 {
            println!("🟢 SimTicker: L1 Hit (QuestDB). Rows: {}", count);
            return self.stream_from_quest(symbol, start_ts, end_ts).await;
        }

        // 2. L3 Fallback: R2
        if let Some(r2) = self.r2_store.clone() {
            println!("🟠 SimTicker: L1 Miss. Triggering L3 Failover (R2 Stream)...");
            return self.stream_from_r2(r2, symbol, start_ts, end_ts).await;
        }

        Err("No data found in QuestDB and R2 failover not configured or failed.".into())
    }

    async fn stream_from_quest(
        self,
        symbol: &str,
        start_ts: i64,
        end_ts: i64
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Tick, Box<dyn std::error::Error + Send + Sync + 'static>>> + Send>>, Box<dyn std::error::Error>> {
        let query = format!(
            "SELECT ts, close, volume 
             FROM ohlcv_1min 
             WHERE symbol = '{}' AND ts BETWEEN '{}' AND '{}' 
             ORDER BY ts ASC",
            symbol,
            to_quest_timestamp(start_ts),
            to_quest_timestamp(end_ts)
        );
        
        // We have to clone client or keep self alive. 
        // Ideally SimTicker shouldn't consume self, but the signature consumes self.
        // Let's adapt to fit the signature.
        let stream = self.client.query_raw(&query, std::iter::empty::<&str>()).await?;

        let tick_stream = stream
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            .map_ok(|row| {
                let ts: SystemTime = row.get(0);
                let price: f64 = row.get(1);
                let volume: f64 = row.get(2);
                let ts_millis = ts.duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as f64;

                Tick {
                    timestamp: ts_millis,
                    price,
                    quantity: volume,
                }
            });

        Ok(Box::pin(tick_stream))
    }

    async fn stream_from_r2(
        self,
        r2: Arc<dyn ObjectStore>,
        symbol: &str,
        start_ts: i64,
        end_ts: i64
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Tick, Box<dyn std::error::Error + Send + Sync + 'static>>> + Send>>, Box<dyn std::error::Error>> {
        
        // We accumulate streams here. 
        // Since we are iterating and creating async logic, we can't easily validly just push to Vec<impl Stream> because the future types might verify.
        // Easier approach: Just use a single stream chain or create a stream of futures.
        
        let week_keys = get_r2_keys(symbol, start_ts, end_ts);
        
        // Stream of keys -> Stream of Ticks
        let r2_arc = r2.clone(); // Arc clone for closure
        
        // We use futures::stream::iter to create a stream of keys
        let key_stream = stream::iter(week_keys);
        
        // We then flat_map this to a stream of Ticks.
        // Since fetching R2 is async (r2.get), we need `then` (map into Future) then `flatten` or `flat_map_unordered`.
        // But we need ORDERED data usually? "ORDER BY ts ASC" is L1.
        // If we process weeks in order, we get order.
        // So `then` (await) -> returns Stream -> `flatten` (sequentially).
        
        let tick_stream = key_stream.then(move |key_str| {
            let r2 = r2_arc.clone();
            async move {
                let path = Path::from(key_str.as_str());
                println!("📦 SimTicker: Fetching Object {}", key_str);
                
                let get_result = match r2.get(&path).await {
                    Ok(r) => r,
                    Err(e) => {
                         println!("⚠️ SimTicker: R2 Object {} miss/error: {}", key_str, e);
                         // Propagate error to halt simulation on data unavailable
                         return stream::iter(vec![Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)]).boxed(); 
                    }
                };

                let bytes = match get_result.bytes().await {
                    Ok(b) => b,
                    Err(e) => {
                        println!("⚠️ SimTicker: R2 Body Error {}: {}", key_str, e);
                         return stream::iter(vec![Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)]).boxed();
                    }
                };
                
                // Wrap in Cursor to provide AsyncSeek/AsyncRead
                let reader = std::io::Cursor::new(bytes);
                let builder = match ParquetRecordBatchStreamBuilder::new(reader).await {
                     Ok(b) => b,
                     Err(e) => {
                         println!("⚠️ SimTicker: Parquet Builder Error {}: {}", key_str, e);
                          return stream::iter(vec![Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)]).boxed();
                     }
                };

                let arrow_schema = builder.schema().clone();
                // Clone metadata to avoid borrowing builder which is moved later
                let metadata = builder.metadata().clone();
                let parquet_schema = metadata.file_metadata().schema_descr();
                
                let target_cols = ["ts", "close", "volume"];
                let indices: Vec<usize> = target_cols.iter().filter_map(|name| {
                    arrow_schema.fields().iter().position(|f| f.name() == *name)
                }).collect();

                let mut stream_builder = builder;
                if indices.len() == 3 {
                    stream_builder = stream_builder.with_projection(parquet::arrow::ProjectionMask::roots(parquet_schema, indices));
                }

                let batch_stream = match stream_builder.build() {
                     Ok(s) => s,
                     Err(e) => {
                         println!("⚠️ SimTicker: Stream Build Error {}: {}", key_str, e);
                          return stream::iter(vec![Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)]).boxed();
                     }
                };

                let mapped_stream = batch_stream
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                    .flat_map(move |batch_res: Result<arrow::array::RecordBatch, Box<dyn std::error::Error + Send + Sync>>| {
                         match batch_res {
                            Ok(batch) => {
                                let ts_col = batch.column_by_name("ts").expect("ts missing");
                                let close_col = batch.column_by_name("close").expect("close missing");
                                let vol_col = batch.column_by_name("volume").expect("volume missing");

                                // We can use arrow::array::cast or downcast. 
                                // Assuming typical schema.
                                let ts_array = ts_col.as_any().downcast_ref::<arrow::array::TimestampMicrosecondArray>();
                                let close_array = close_col.as_any().downcast_ref::<arrow::array::Float64Array>();
                                let vol_array = vol_col.as_any().downcast_ref::<arrow::array::Float64Array>();
                                
                                if let (Some(ts), Some(close), Some(vol)) = (ts_array, close_array, vol_array) {
                                    let mut ticks: Vec<Result<Tick, Box<dyn std::error::Error + Send + Sync>>> = Vec::with_capacity(batch.num_rows());
                                    for i in 0..batch.num_rows() {
                                        ticks.push(Ok(Tick {
                                            timestamp: (ts.value(i) / 1000) as f64,
                                            price: close.value(i),
                                            quantity: vol.value(i),
                                        }));
                                    }
                                    stream::iter(ticks)
                                } else {
                                     stream::iter(vec![Err("Column Type Mismatch in Parquet".into())])
                                }
                            }
                            Err(e) => stream::iter(vec![Err(e)]),
                         }
                    });
                
                mapped_stream.boxed()
            }
        }).flatten();

        Ok(Box::pin(tick_stream))
    }
}

fn to_quest_timestamp(millis: i64) -> String {
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(millis / 1000, ((millis % 1000) * 1_000_000) as u32).unwrap();
    dt.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string()
}

fn get_r2_keys(symbol: &str, start_ts: i64, end_ts: i64) -> Vec<String> {
    let start_dt = DateTime::<Utc>::from_timestamp(start_ts / 1000, 0).unwrap();
    let end_dt = DateTime::<Utc>::from_timestamp(end_ts / 1000, 0).unwrap();
    
    let mut weeks = Vec::new();
    let mut curr = start_dt;

    while curr <= end_dt {
        // Format: symbol/YYYY_Www.parquet
        // Iso week
        let iso_year = curr.iso_week().year();
        let iso_week = curr.iso_week().week();
        let key = format!("{}/{:04}_W{:02}.parquet", symbol, iso_year, iso_week);
        if !weeks.contains(&key) {
           weeks.push(key);
        }
        curr = curr + Duration::days(7);
    }
    // ensure end date week is covered (loop logic covers if step is < range, but boundary check)
    let end_key = format!("{}/{:04}_W{:02}.parquet", symbol, end_dt.iso_week().year(), end_dt.iso_week().week());
    if !weeks.contains(&end_key) {
        weeks.push(end_key);
    }

    weeks
}

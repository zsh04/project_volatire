import * as jspb from 'google-protobuf'



export class PhysicsResponse extends jspb.Message {
  getPrice(): number;
  setPrice(value: number): PhysicsResponse;

  getVelocity(): number;
  setVelocity(value: number): PhysicsResponse;

  getAcceleration(): number;
  setAcceleration(value: number): PhysicsResponse;

  getJerk(): number;
  setJerk(value: number): PhysicsResponse;

  getEntropy(): number;
  setEntropy(value: number): PhysicsResponse;

  getEfficiencyIndex(): number;
  setEfficiencyIndex(value: number): PhysicsResponse;

  getTimestamp(): number;
  setTimestamp(value: number): PhysicsResponse;

  getUnrealizedPnl(): number;
  setUnrealizedPnl(value: number): PhysicsResponse;

  getEquity(): number;
  setEquity(value: number): PhysicsResponse;

  getBalance(): number;
  setBalance(value: number): PhysicsResponse;

  getGemmaTokensPerSec(): number;
  setGemmaTokensPerSec(value: number): PhysicsResponse;

  getGemmaLatencyMs(): number;
  setGemmaLatencyMs(value: number): PhysicsResponse;

  getStaircaseTier(): number;
  setStaircaseTier(value: number): PhysicsResponse;

  getStaircaseProgress(): number;
  setStaircaseProgress(value: number): PhysicsResponse;

  getAuditDrift(): number;
  setAuditDrift(value: number): PhysicsResponse;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): PhysicsResponse.AsObject;
  static toObject(includeInstance: boolean, msg: PhysicsResponse): PhysicsResponse.AsObject;
  static serializeBinaryToWriter(message: PhysicsResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): PhysicsResponse;
  static deserializeBinaryFromReader(message: PhysicsResponse, reader: jspb.BinaryReader): PhysicsResponse;
}

export namespace PhysicsResponse {
  export type AsObject = {
    price: number,
    velocity: number,
    acceleration: number,
    jerk: number,
    entropy: number,
    efficiencyIndex: number,
    timestamp: number,
    unrealizedPnl: number,
    equity: number,
    balance: number,
    gemmaTokensPerSec: number,
    gemmaLatencyMs: number,
    staircaseTier: number,
    staircaseProgress: number,
    auditDrift: number,
  }
}

export class OODAResponse extends jspb.Message {
  getPhysics(): PhysicsResponse | undefined;
  setPhysics(value?: PhysicsResponse): OODAResponse;
  hasPhysics(): boolean;
  clearPhysics(): OODAResponse;

  getSentimentScore(): number;
  setSentimentScore(value: number): OODAResponse;
  hasSentimentScore(): boolean;
  clearSentimentScore(): OODAResponse;

  getNearestRegime(): string;
  setNearestRegime(value: string): OODAResponse;
  hasNearestRegime(): boolean;
  clearNearestRegime(): OODAResponse;

  getDecision(): string;
  setDecision(value: string): OODAResponse;

  getWeightsMap(): jspb.Map<string, number>;
  clearWeightsMap(): OODAResponse;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): OODAResponse.AsObject;
  static toObject(includeInstance: boolean, msg: OODAResponse): OODAResponse.AsObject;
  static serializeBinaryToWriter(message: OODAResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): OODAResponse;
  static deserializeBinaryFromReader(message: OODAResponse, reader: jspb.BinaryReader): OODAResponse;
}

export namespace OODAResponse {
  export type AsObject = {
    physics?: PhysicsResponse.AsObject,
    sentimentScore?: number,
    nearestRegime?: string,
    decision: string,
    weightsMap: Array<[string, number]>,
  }

  export enum SentimentScoreCase { 
    _SENTIMENT_SCORE_NOT_SET = 0,
    SENTIMENT_SCORE = 2,
  }

  export enum NearestRegimeCase { 
    _NEAREST_REGIME_NOT_SET = 0,
    NEAREST_REGIME = 3,
  }
}

export class VetoRequest extends jspb.Message {
  getReason(): string;
  setReason(value: string): VetoRequest;

  getOperator(): string;
  setOperator(value: string): VetoRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): VetoRequest.AsObject;
  static toObject(includeInstance: boolean, msg: VetoRequest): VetoRequest.AsObject;
  static serializeBinaryToWriter(message: VetoRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): VetoRequest;
  static deserializeBinaryFromReader(message: VetoRequest, reader: jspb.BinaryReader): VetoRequest;
}

export namespace VetoRequest {
  export type AsObject = {
    reason: string,
    operator: string,
  }
}

export class DemoteRequest extends jspb.Message {
  getReason(): string;
  setReason(value: string): DemoteRequest;

  getTargetLevel(): string;
  setTargetLevel(value: string): DemoteRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DemoteRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DemoteRequest): DemoteRequest.AsObject;
  static serializeBinaryToWriter(message: DemoteRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DemoteRequest;
  static deserializeBinaryFromReader(message: DemoteRequest, reader: jspb.BinaryReader): DemoteRequest;
}

export namespace DemoteRequest {
  export type AsObject = {
    reason: string,
    targetLevel: string,
  }
}

export class RatchetRequest extends jspb.Message {
  getLevel(): RatchetRequest.Level;
  setLevel(value: RatchetRequest.Level): RatchetRequest;

  getReason(): string;
  setReason(value: string): RatchetRequest;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RatchetRequest.AsObject;
  static toObject(includeInstance: boolean, msg: RatchetRequest): RatchetRequest.AsObject;
  static serializeBinaryToWriter(message: RatchetRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RatchetRequest;
  static deserializeBinaryFromReader(message: RatchetRequest, reader: jspb.BinaryReader): RatchetRequest;
}

export namespace RatchetRequest {
  export type AsObject = {
    level: RatchetRequest.Level,
    reason: string,
  }

  export enum Level { 
    IDLE = 0,
    TIGHTEN = 1,
    FREEZE = 2,
    KILL = 3,
  }
}

export class ConfigPayload extends jspb.Message {
  getKey(): string;
  setKey(value: string): ConfigPayload;

  getValue(): number;
  setValue(value: number): ConfigPayload;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ConfigPayload.AsObject;
  static toObject(includeInstance: boolean, msg: ConfigPayload): ConfigPayload.AsObject;
  static serializeBinaryToWriter(message: ConfigPayload, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ConfigPayload;
  static deserializeBinaryFromReader(message: ConfigPayload, reader: jspb.BinaryReader): ConfigPayload;
}

export namespace ConfigPayload {
  export type AsObject = {
    key: string,
    value: number,
  }
}

export class Heartbeat extends jspb.Message {
  getTimestamp(): number;
  setTimestamp(value: number): Heartbeat;

  getBrainConnected(): boolean;
  setBrainConnected(value: boolean): Heartbeat;

  getEfficiencyIndex(): number;
  setEfficiencyIndex(value: number): Heartbeat;

  getPRiemann(): number;
  setPRiemann(value: number): Heartbeat;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Heartbeat.AsObject;
  static toObject(includeInstance: boolean, msg: Heartbeat): Heartbeat.AsObject;
  static serializeBinaryToWriter(message: Heartbeat, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Heartbeat;
  static deserializeBinaryFromReader(message: Heartbeat, reader: jspb.BinaryReader): Heartbeat;
}

export namespace Heartbeat {
  export type AsObject = {
    timestamp: number,
    brainConnected: boolean,
    efficiencyIndex: number,
    pRiemann: number,
  }
}

export class Ack extends jspb.Message {
  getSuccess(): boolean;
  setSuccess(value: boolean): Ack;

  getMessage(): string;
  setMessage(value: string): Ack;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Ack.AsObject;
  static toObject(includeInstance: boolean, msg: Ack): Ack.AsObject;
  static serializeBinaryToWriter(message: Ack, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Ack;
  static deserializeBinaryFromReader(message: Ack, reader: jspb.BinaryReader): Ack;
}

export namespace Ack {
  export type AsObject = {
    success: boolean,
    message: string,
  }
}

export class Empty extends jspb.Message {
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Empty.AsObject;
  static toObject(includeInstance: boolean, msg: Empty): Empty.AsObject;
  static serializeBinaryToWriter(message: Empty, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Empty;
  static deserializeBinaryFromReader(message: Empty, reader: jspb.BinaryReader): Empty;
}

export namespace Empty {
  export type AsObject = {
  }
}


// package: reflex
// file: reflex.proto

import * as jspb from "google-protobuf";

export class TickHistoryRequest extends jspb.Message {
  getSymbol(): string;
  setSymbol(value: string): void;

  getStartTime(): number;
  setStartTime(value: number): void;

  getEndTime(): number;
  setEndTime(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): TickHistoryRequest.AsObject;
  static toObject(includeInstance: boolean, msg: TickHistoryRequest): TickHistoryRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: TickHistoryRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): TickHistoryRequest;
  static deserializeBinaryFromReader(message: TickHistoryRequest, reader: jspb.BinaryReader): TickHistoryRequest;
}

export namespace TickHistoryRequest {
  export type AsObject = {
    symbol: string,
    startTime: number,
    endTime: number,
  }
}

export class PhysicsResponse extends jspb.Message {
  getPrice(): number;
  setPrice(value: number): void;

  getVelocity(): number;
  setVelocity(value: number): void;

  getAcceleration(): number;
  setAcceleration(value: number): void;

  getJerk(): number;
  setJerk(value: number): void;

  getEntropy(): number;
  setEntropy(value: number): void;

  getEfficiencyIndex(): number;
  setEfficiencyIndex(value: number): void;

  getTimestamp(): number;
  setTimestamp(value: number): void;

  getSequenceId(): number;
  setSequenceId(value: number): void;

  getUnrealizedPnl(): number;
  setUnrealizedPnl(value: number): void;

  getEquity(): number;
  setEquity(value: number): void;

  getBalance(): number;
  setBalance(value: number): void;

  getRealizedPnl(): number;
  setRealizedPnl(value: number): void;

  getBtcPosition(): number;
  setBtcPosition(value: number): void;

  getGemmaTokensPerSec(): number;
  setGemmaTokensPerSec(value: number): void;

  getGemmaLatencyMs(): number;
  setGemmaLatencyMs(value: number): void;

  getStaircaseTier(): number;
  setStaircaseTier(value: number): void;

  getStaircaseProgress(): number;
  setStaircaseProgress(value: number): void;

  getAuditDrift(): number;
  setAuditDrift(value: number): void;

  getSystemLatencyUs(): number;
  setSystemLatencyUs(value: number): void;

  getSystemJitterUs(): number;
  setSystemJitterUs(value: number): void;

  getVitalityStatus(): string;
  setVitalityStatus(value: string): void;

  clearReasoningTraceList(): void;
  getReasoningTraceList(): Array<ReasoningStep>;
  setReasoningTraceList(value: Array<ReasoningStep>): void;
  addReasoningTrace(value?: ReasoningStep, index?: number): ReasoningStep;

  getIgnitionStatus(): string;
  setIgnitionStatus(value: string): void;

  getSystemSanityScore(): number;
  setSystemSanityScore(value: number): void;

  clearPositionsList(): void;
  getPositionsList(): Array<PositionState>;
  setPositionsList(value: Array<PositionState>): void;
  addPositions(value?: PositionState, index?: number): PositionState;

  clearOrdersList(): void;
  getOrdersList(): Array<OrderState>;
  setOrdersList(value: Array<OrderState>): void;
  addOrders(value?: OrderState, index?: number): OrderState;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): PhysicsResponse.AsObject;
  static toObject(includeInstance: boolean, msg: PhysicsResponse): PhysicsResponse.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
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
    sequenceId: number,
    unrealizedPnl: number,
    equity: number,
    balance: number,
    realizedPnl: number,
    btcPosition: number,
    gemmaTokensPerSec: number,
    gemmaLatencyMs: number,
    staircaseTier: number,
    staircaseProgress: number,
    auditDrift: number,
    systemLatencyUs: number,
    systemJitterUs: number,
    vitalityStatus: string,
    reasoningTraceList: Array<ReasoningStep.AsObject>,
    ignitionStatus: string,
    systemSanityScore: number,
    positionsList: Array<PositionState.AsObject>,
    ordersList: Array<OrderState.AsObject>,
  }
}

export class PositionState extends jspb.Message {
  getSymbol(): string;
  setSymbol(value: string): void;

  getNetSize(): number;
  setNetSize(value: number): void;

  getAvgEntryPrice(): number;
  setAvgEntryPrice(value: number): void;

  getUnrealizedPnl(): number;
  setUnrealizedPnl(value: number): void;

  getEntryTimestamp(): number;
  setEntryTimestamp(value: number): void;

  getCurrentPrice(): number;
  setCurrentPrice(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): PositionState.AsObject;
  static toObject(includeInstance: boolean, msg: PositionState): PositionState.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: PositionState, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): PositionState;
  static deserializeBinaryFromReader(message: PositionState, reader: jspb.BinaryReader): PositionState;
}

export namespace PositionState {
  export type AsObject = {
    symbol: string,
    netSize: number,
    avgEntryPrice: number,
    unrealizedPnl: number,
    entryTimestamp: number,
    currentPrice: number,
  }
}

export class OrderState extends jspb.Message {
  getOrderId(): string;
  setOrderId(value: string): void;

  getSymbol(): string;
  setSymbol(value: string): void;

  getSide(): string;
  setSide(value: string): void;

  getQuantity(): number;
  setQuantity(value: number): void;

  getLimitPrice(): number;
  setLimitPrice(value: number): void;

  getStatus(): string;
  setStatus(value: string): void;

  getTimestamp(): number;
  setTimestamp(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): OrderState.AsObject;
  static toObject(includeInstance: boolean, msg: OrderState): OrderState.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: OrderState, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): OrderState;
  static deserializeBinaryFromReader(message: OrderState, reader: jspb.BinaryReader): OrderState;
}

export namespace OrderState {
  export type AsObject = {
    orderId: string,
    symbol: string,
    side: string,
    quantity: number,
    limitPrice: number,
    status: string,
    timestamp: number,
  }
}

export class ReasoningStep extends jspb.Message {
  getId(): string;
  setId(value: string): void;

  getContent(): string;
  setContent(value: string): void;

  getProbability(): number;
  setProbability(value: number): void;

  getType(): string;
  setType(value: string): void;

  getTimestamp(): number;
  setTimestamp(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ReasoningStep.AsObject;
  static toObject(includeInstance: boolean, msg: ReasoningStep): ReasoningStep.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: ReasoningStep, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ReasoningStep;
  static deserializeBinaryFromReader(message: ReasoningStep, reader: jspb.BinaryReader): ReasoningStep;
}

export namespace ReasoningStep {
  export type AsObject = {
    id: string,
    content: string,
    probability: number,
    type: string,
    timestamp: number,
  }
}

export class OODAResponse extends jspb.Message {
  hasPhysics(): boolean;
  clearPhysics(): void;
  getPhysics(): PhysicsResponse | undefined;
  setPhysics(value?: PhysicsResponse): void;

  hasSentimentScore(): boolean;
  clearSentimentScore(): void;
  getSentimentScore(): number;
  setSentimentScore(value: number): void;

  hasNearestRegime(): boolean;
  clearNearestRegime(): void;
  getNearestRegime(): string;
  setNearestRegime(value: string): void;

  getDecision(): string;
  setDecision(value: string): void;

  getWeightsMap(): jspb.Map<string, number>;
  clearWeightsMap(): void;
  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): OODAResponse.AsObject;
  static toObject(includeInstance: boolean, msg: OODAResponse): OODAResponse.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: OODAResponse, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): OODAResponse;
  static deserializeBinaryFromReader(message: OODAResponse, reader: jspb.BinaryReader): OODAResponse;
}

export namespace OODAResponse {
  export type AsObject = {
    physics?: PhysicsResponse.AsObject,
    sentimentScore: number,
    nearestRegime: string,
    decision: string,
    weightsMap: Array<[string, number]>,
  }
}

export class VetoRequest extends jspb.Message {
  getReason(): string;
  setReason(value: string): void;

  getOperator(): string;
  setOperator(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): VetoRequest.AsObject;
  static toObject(includeInstance: boolean, msg: VetoRequest): VetoRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
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
  setReason(value: string): void;

  getTargetLevel(): string;
  setTargetLevel(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DemoteRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DemoteRequest): DemoteRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
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
  getLevel(): RatchetRequest.LevelMap[keyof RatchetRequest.LevelMap];
  setLevel(value: RatchetRequest.LevelMap[keyof RatchetRequest.LevelMap]): void;

  getReason(): string;
  setReason(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RatchetRequest.AsObject;
  static toObject(includeInstance: boolean, msg: RatchetRequest): RatchetRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: RatchetRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RatchetRequest;
  static deserializeBinaryFromReader(message: RatchetRequest, reader: jspb.BinaryReader): RatchetRequest;
}

export namespace RatchetRequest {
  export type AsObject = {
    level: RatchetRequest.LevelMap[keyof RatchetRequest.LevelMap],
    reason: string,
  }

  export interface LevelMap {
    IDLE: 0;
    TIGHTEN: 1;
    FREEZE: 2;
    KILL: 3;
  }

  export const Level: LevelMap;
}

export class LegislativeUpdate extends jspb.Message {
  getBias(): string;
  setBias(value: string): void;

  getAggression(): number;
  setAggression(value: number): void;

  getMakerOnly(): boolean;
  setMakerOnly(value: boolean): void;

  getHibernation(): boolean;
  setHibernation(value: boolean): void;

  getSnapToBreakeven(): boolean;
  setSnapToBreakeven(value: boolean): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): LegislativeUpdate.AsObject;
  static toObject(includeInstance: boolean, msg: LegislativeUpdate): LegislativeUpdate.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: LegislativeUpdate, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): LegislativeUpdate;
  static deserializeBinaryFromReader(message: LegislativeUpdate, reader: jspb.BinaryReader): LegislativeUpdate;
}

export namespace LegislativeUpdate {
  export type AsObject = {
    bias: string,
    aggression: number,
    makerOnly: boolean,
    hibernation: boolean,
    snapToBreakeven: boolean,
  }
}

export class CancelOrderRequest extends jspb.Message {
  getOrderId(): string;
  setOrderId(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CancelOrderRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CancelOrderRequest): CancelOrderRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: CancelOrderRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CancelOrderRequest;
  static deserializeBinaryFromReader(message: CancelOrderRequest, reader: jspb.BinaryReader): CancelOrderRequest;
}

export namespace CancelOrderRequest {
  export type AsObject = {
    orderId: string,
  }
}

export class ClosePositionRequest extends jspb.Message {
  getSymbol(): string;
  setSymbol(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ClosePositionRequest.AsObject;
  static toObject(includeInstance: boolean, msg: ClosePositionRequest): ClosePositionRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: ClosePositionRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): ClosePositionRequest;
  static deserializeBinaryFromReader(message: ClosePositionRequest, reader: jspb.BinaryReader): ClosePositionRequest;
}

export namespace ClosePositionRequest {
  export type AsObject = {
    symbol: string,
  }
}

export class ConfigPayload extends jspb.Message {
  getKey(): string;
  setKey(value: string): void;

  getValue(): number;
  setValue(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): ConfigPayload.AsObject;
  static toObject(includeInstance: boolean, msg: ConfigPayload): ConfigPayload.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
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
  setTimestamp(value: number): void;

  getBrainConnected(): boolean;
  setBrainConnected(value: boolean): void;

  getEfficiencyIndex(): number;
  setEfficiencyIndex(value: number): void;

  getPRiemann(): number;
  setPRiemann(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Heartbeat.AsObject;
  static toObject(includeInstance: boolean, msg: Heartbeat): Heartbeat.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
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
  setSuccess(value: boolean): void;

  getMessage(): string;
  setMessage(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Ack.AsObject;
  static toObject(includeInstance: boolean, msg: Ack): Ack.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
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
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: Empty, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Empty;
  static deserializeBinaryFromReader(message: Empty, reader: jspb.BinaryReader): Empty;
}

export namespace Empty {
  export type AsObject = {
  }
}


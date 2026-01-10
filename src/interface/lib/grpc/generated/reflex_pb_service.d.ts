// package: reflex
// file: reflex.proto

import * as reflex_pb from "./reflex_pb";
import {grpc} from "@improbable-eng/grpc-web";

type ReflexServiceGetPhysics = {
  readonly methodName: string;
  readonly service: typeof ReflexService;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof reflex_pb.Empty;
  readonly responseType: typeof reflex_pb.PhysicsResponse;
};

type ReflexServiceGetOODA = {
  readonly methodName: string;
  readonly service: typeof ReflexService;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof reflex_pb.Empty;
  readonly responseType: typeof reflex_pb.OODAResponse;
};

type ReflexServiceTriggerVeto = {
  readonly methodName: string;
  readonly service: typeof ReflexService;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof reflex_pb.VetoRequest;
  readonly responseType: typeof reflex_pb.Ack;
};

type ReflexServiceDemoteProvisional = {
  readonly methodName: string;
  readonly service: typeof ReflexService;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof reflex_pb.DemoteRequest;
  readonly responseType: typeof reflex_pb.Ack;
};

type ReflexServiceGetTickHistory = {
  readonly methodName: string;
  readonly service: typeof ReflexService;
  readonly requestStream: false;
  readonly responseStream: true;
  readonly requestType: typeof reflex_pb.TickHistoryRequest;
  readonly responseType: typeof reflex_pb.PhysicsResponse;
};

type ReflexServiceInitiateIgnition = {
  readonly methodName: string;
  readonly service: typeof ReflexService;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof reflex_pb.Empty;
  readonly responseType: typeof reflex_pb.Ack;
};

type ReflexServiceUpdateLegislation = {
  readonly methodName: string;
  readonly service: typeof ReflexService;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof reflex_pb.LegislativeUpdate;
  readonly responseType: typeof reflex_pb.Ack;
};

type ReflexServiceCancelOrder = {
  readonly methodName: string;
  readonly service: typeof ReflexService;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof reflex_pb.CancelOrderRequest;
  readonly responseType: typeof reflex_pb.Ack;
};

type ReflexServiceClosePosition = {
  readonly methodName: string;
  readonly service: typeof ReflexService;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof reflex_pb.ClosePositionRequest;
  readonly responseType: typeof reflex_pb.Ack;
};

type ReflexServiceTriggerRatchet = {
  readonly methodName: string;
  readonly service: typeof ReflexService;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof reflex_pb.RatchetRequest;
  readonly responseType: typeof reflex_pb.Ack;
};

type ReflexServiceUpdateConfig = {
  readonly methodName: string;
  readonly service: typeof ReflexService;
  readonly requestStream: false;
  readonly responseStream: false;
  readonly requestType: typeof reflex_pb.ConfigPayload;
  readonly responseType: typeof reflex_pb.Ack;
};

type ReflexServiceGetStream = {
  readonly methodName: string;
  readonly service: typeof ReflexService;
  readonly requestStream: false;
  readonly responseStream: true;
  readonly requestType: typeof reflex_pb.Empty;
  readonly responseType: typeof reflex_pb.PhysicsResponse;
};

export class ReflexService {
  static readonly serviceName: string;
  static readonly GetPhysics: ReflexServiceGetPhysics;
  static readonly GetOODA: ReflexServiceGetOODA;
  static readonly TriggerVeto: ReflexServiceTriggerVeto;
  static readonly DemoteProvisional: ReflexServiceDemoteProvisional;
  static readonly GetTickHistory: ReflexServiceGetTickHistory;
  static readonly InitiateIgnition: ReflexServiceInitiateIgnition;
  static readonly UpdateLegislation: ReflexServiceUpdateLegislation;
  static readonly CancelOrder: ReflexServiceCancelOrder;
  static readonly ClosePosition: ReflexServiceClosePosition;
  static readonly TriggerRatchet: ReflexServiceTriggerRatchet;
  static readonly UpdateConfig: ReflexServiceUpdateConfig;
  static readonly GetStream: ReflexServiceGetStream;
}

export type ServiceError = { message: string, code: number; metadata: grpc.Metadata }
export type Status = { details: string, code: number; metadata: grpc.Metadata }

interface UnaryResponse {
  cancel(): void;
}
interface ResponseStream<T> {
  cancel(): void;
  on(type: 'data', handler: (message: T) => void): ResponseStream<T>;
  on(type: 'end', handler: (status?: Status) => void): ResponseStream<T>;
  on(type: 'status', handler: (status: Status) => void): ResponseStream<T>;
}
interface RequestStream<T> {
  write(message: T): RequestStream<T>;
  end(): void;
  cancel(): void;
  on(type: 'end', handler: (status?: Status) => void): RequestStream<T>;
  on(type: 'status', handler: (status: Status) => void): RequestStream<T>;
}
interface BidirectionalStream<ReqT, ResT> {
  write(message: ReqT): BidirectionalStream<ReqT, ResT>;
  end(): void;
  cancel(): void;
  on(type: 'data', handler: (message: ResT) => void): BidirectionalStream<ReqT, ResT>;
  on(type: 'end', handler: (status?: Status) => void): BidirectionalStream<ReqT, ResT>;
  on(type: 'status', handler: (status: Status) => void): BidirectionalStream<ReqT, ResT>;
}

export class ReflexServiceClient {
  readonly serviceHost: string;

  constructor(serviceHost: string, options?: grpc.RpcOptions);
  getPhysics(
    requestMessage: reflex_pb.Empty,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.PhysicsResponse|null) => void
  ): UnaryResponse;
  getPhysics(
    requestMessage: reflex_pb.Empty,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.PhysicsResponse|null) => void
  ): UnaryResponse;
  getOODA(
    requestMessage: reflex_pb.Empty,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.OODAResponse|null) => void
  ): UnaryResponse;
  getOODA(
    requestMessage: reflex_pb.Empty,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.OODAResponse|null) => void
  ): UnaryResponse;
  triggerVeto(
    requestMessage: reflex_pb.VetoRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  triggerVeto(
    requestMessage: reflex_pb.VetoRequest,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  demoteProvisional(
    requestMessage: reflex_pb.DemoteRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  demoteProvisional(
    requestMessage: reflex_pb.DemoteRequest,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  getTickHistory(requestMessage: reflex_pb.TickHistoryRequest, metadata?: grpc.Metadata): ResponseStream<reflex_pb.PhysicsResponse>;
  initiateIgnition(
    requestMessage: reflex_pb.Empty,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  initiateIgnition(
    requestMessage: reflex_pb.Empty,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  updateLegislation(
    requestMessage: reflex_pb.LegislativeUpdate,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  updateLegislation(
    requestMessage: reflex_pb.LegislativeUpdate,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  cancelOrder(
    requestMessage: reflex_pb.CancelOrderRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  cancelOrder(
    requestMessage: reflex_pb.CancelOrderRequest,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  closePosition(
    requestMessage: reflex_pb.ClosePositionRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  closePosition(
    requestMessage: reflex_pb.ClosePositionRequest,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  triggerRatchet(
    requestMessage: reflex_pb.RatchetRequest,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  triggerRatchet(
    requestMessage: reflex_pb.RatchetRequest,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  updateConfig(
    requestMessage: reflex_pb.ConfigPayload,
    metadata: grpc.Metadata,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  updateConfig(
    requestMessage: reflex_pb.ConfigPayload,
    callback: (error: ServiceError|null, responseMessage: reflex_pb.Ack|null) => void
  ): UnaryResponse;
  getStream(requestMessage: reflex_pb.Empty, metadata?: grpc.Metadata): ResponseStream<reflex_pb.PhysicsResponse>;
}

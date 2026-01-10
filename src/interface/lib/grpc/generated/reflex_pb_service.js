// package: reflex
// file: reflex.proto

var reflex_pb = require("./reflex_pb");
var grpc = require("@improbable-eng/grpc-web").grpc;

var ReflexService = (function () {
  function ReflexService() {}
  ReflexService.serviceName = "reflex.ReflexService";
  return ReflexService;
}());

ReflexService.GetPhysics = {
  methodName: "GetPhysics",
  service: ReflexService,
  requestStream: false,
  responseStream: false,
  requestType: reflex_pb.Empty,
  responseType: reflex_pb.PhysicsResponse
};

ReflexService.GetOODA = {
  methodName: "GetOODA",
  service: ReflexService,
  requestStream: false,
  responseStream: false,
  requestType: reflex_pb.Empty,
  responseType: reflex_pb.OODAResponse
};

ReflexService.TriggerVeto = {
  methodName: "TriggerVeto",
  service: ReflexService,
  requestStream: false,
  responseStream: false,
  requestType: reflex_pb.VetoRequest,
  responseType: reflex_pb.Ack
};

ReflexService.DemoteProvisional = {
  methodName: "DemoteProvisional",
  service: ReflexService,
  requestStream: false,
  responseStream: false,
  requestType: reflex_pb.DemoteRequest,
  responseType: reflex_pb.Ack
};

ReflexService.GetTickHistory = {
  methodName: "GetTickHistory",
  service: ReflexService,
  requestStream: false,
  responseStream: true,
  requestType: reflex_pb.TickHistoryRequest,
  responseType: reflex_pb.PhysicsResponse
};

ReflexService.InitiateIgnition = {
  methodName: "InitiateIgnition",
  service: ReflexService,
  requestStream: false,
  responseStream: false,
  requestType: reflex_pb.Empty,
  responseType: reflex_pb.Ack
};

ReflexService.UpdateLegislation = {
  methodName: "UpdateLegislation",
  service: ReflexService,
  requestStream: false,
  responseStream: false,
  requestType: reflex_pb.LegislativeUpdate,
  responseType: reflex_pb.Ack
};

ReflexService.CancelOrder = {
  methodName: "CancelOrder",
  service: ReflexService,
  requestStream: false,
  responseStream: false,
  requestType: reflex_pb.CancelOrderRequest,
  responseType: reflex_pb.Ack
};

ReflexService.ClosePosition = {
  methodName: "ClosePosition",
  service: ReflexService,
  requestStream: false,
  responseStream: false,
  requestType: reflex_pb.ClosePositionRequest,
  responseType: reflex_pb.Ack
};

ReflexService.TriggerRatchet = {
  methodName: "TriggerRatchet",
  service: ReflexService,
  requestStream: false,
  responseStream: false,
  requestType: reflex_pb.RatchetRequest,
  responseType: reflex_pb.Ack
};

ReflexService.UpdateConfig = {
  methodName: "UpdateConfig",
  service: ReflexService,
  requestStream: false,
  responseStream: false,
  requestType: reflex_pb.ConfigPayload,
  responseType: reflex_pb.Ack
};

ReflexService.GetStream = {
  methodName: "GetStream",
  service: ReflexService,
  requestStream: false,
  responseStream: true,
  requestType: reflex_pb.Empty,
  responseType: reflex_pb.PhysicsResponse
};

exports.ReflexService = ReflexService;

function ReflexServiceClient(serviceHost, options) {
  this.serviceHost = serviceHost;
  this.options = options || {};
}

ReflexServiceClient.prototype.getPhysics = function getPhysics(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(ReflexService.GetPhysics, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ReflexServiceClient.prototype.getOODA = function getOODA(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(ReflexService.GetOODA, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ReflexServiceClient.prototype.triggerVeto = function triggerVeto(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(ReflexService.TriggerVeto, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ReflexServiceClient.prototype.demoteProvisional = function demoteProvisional(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(ReflexService.DemoteProvisional, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ReflexServiceClient.prototype.getTickHistory = function getTickHistory(requestMessage, metadata) {
  var listeners = {
    data: [],
    end: [],
    status: []
  };
  var client = grpc.invoke(ReflexService.GetTickHistory, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onMessage: function (responseMessage) {
      listeners.data.forEach(function (handler) {
        handler(responseMessage);
      });
    },
    onEnd: function (status, statusMessage, trailers) {
      listeners.status.forEach(function (handler) {
        handler({ code: status, details: statusMessage, metadata: trailers });
      });
      listeners.end.forEach(function (handler) {
        handler({ code: status, details: statusMessage, metadata: trailers });
      });
      listeners = null;
    }
  });
  return {
    on: function (type, handler) {
      listeners[type].push(handler);
      return this;
    },
    cancel: function () {
      listeners = null;
      client.close();
    }
  };
};

ReflexServiceClient.prototype.initiateIgnition = function initiateIgnition(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(ReflexService.InitiateIgnition, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ReflexServiceClient.prototype.updateLegislation = function updateLegislation(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(ReflexService.UpdateLegislation, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ReflexServiceClient.prototype.cancelOrder = function cancelOrder(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(ReflexService.CancelOrder, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ReflexServiceClient.prototype.closePosition = function closePosition(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(ReflexService.ClosePosition, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ReflexServiceClient.prototype.triggerRatchet = function triggerRatchet(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(ReflexService.TriggerRatchet, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ReflexServiceClient.prototype.updateConfig = function updateConfig(requestMessage, metadata, callback) {
  if (arguments.length === 2) {
    callback = arguments[1];
  }
  var client = grpc.unary(ReflexService.UpdateConfig, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onEnd: function (response) {
      if (callback) {
        if (response.status !== grpc.Code.OK) {
          var err = new Error(response.statusMessage);
          err.code = response.status;
          err.metadata = response.trailers;
          callback(err, null);
        } else {
          callback(null, response.message);
        }
      }
    }
  });
  return {
    cancel: function () {
      callback = null;
      client.close();
    }
  };
};

ReflexServiceClient.prototype.getStream = function getStream(requestMessage, metadata) {
  var listeners = {
    data: [],
    end: [],
    status: []
  };
  var client = grpc.invoke(ReflexService.GetStream, {
    request: requestMessage,
    host: this.serviceHost,
    metadata: metadata,
    transport: this.options.transport,
    debug: this.options.debug,
    onMessage: function (responseMessage) {
      listeners.data.forEach(function (handler) {
        handler(responseMessage);
      });
    },
    onEnd: function (status, statusMessage, trailers) {
      listeners.status.forEach(function (handler) {
        handler({ code: status, details: statusMessage, metadata: trailers });
      });
      listeners.end.forEach(function (handler) {
        handler({ code: status, details: statusMessage, metadata: trailers });
      });
      listeners = null;
    }
  });
  return {
    on: function (type, handler) {
      listeners[type].push(handler);
      return this;
    },
    cancel: function () {
      listeners = null;
      client.close();
    }
  };
};

exports.ReflexServiceClient = ReflexServiceClient;

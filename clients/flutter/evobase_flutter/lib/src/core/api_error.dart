import '../models/api_response.dart';

/// Base API exception
abstract class ApiException implements Exception {

  const ApiException({
    required this.code,
    required this.message,
    this.field,
  });

  factory ApiException.fromErrorDetail(ErrorDetail detail) {
    switch (detail.code) {
      case 'UNAUTHORIZED':
        return UnauthorizedException(
          message: detail.message,
          field: detail.field,
        );
      case 'FORBIDDEN':
        return ForbiddenException(
          message: detail.message,
          field: detail.field,
        );
      case 'NOT_FOUND':
        return NotFoundException(
          message: detail.message,
          field: detail.field,
        );
      case 'VALIDATION_ERROR':
        return ValidationException(
          message: detail.message,
          field: detail.field,
        );
      case 'CONFLICT':
        return ConflictException(
          message: detail.message,
          field: detail.field,
        );
      case 'INTERNAL_ERROR':
        return InternalServerException(
          message: detail.message,
          field: detail.field,
        );
      default:
        return UnknownApiException(
          code: detail.code,
          message: detail.message,
          field: detail.field,
        );
    }
  }
  final String code;
  final String message;
  final String? field;

  @override
  String toString() =>
      'ApiException($code): $message${field != null ? ' [field: $field]' : ''}';
}

class UnauthorizedException extends ApiException {
  UnauthorizedException({required super.message, super.field})
      : super(code: 'UNAUTHORIZED');
}

class ForbiddenException extends ApiException {
  ForbiddenException({required super.message, super.field})
      : super(code: 'FORBIDDEN');
}

class NotFoundException extends ApiException {
  NotFoundException({required super.message, super.field})
      : super(code: 'NOT_FOUND');
}

class ValidationException extends ApiException {
  ValidationException({required super.message, super.field})
      : super(code: 'VALIDATION_ERROR');
}

class ConflictException extends ApiException {
  ConflictException({required super.message, super.field})
      : super(code: 'CONFLICT');
}

class InternalServerException extends ApiException {
  InternalServerException({required super.message, super.field})
      : super(code: 'INTERNAL_ERROR');
}

class UnknownApiException extends ApiException {
  UnknownApiException({
    required super.code,
    required super.message,
    super.field,
  });
}

class NetworkException extends ApiException {
  NetworkException({required super.message}) : super(code: 'NETWORK_ERROR');
}

class TimeoutException extends ApiException {
  TimeoutException({required super.message}) : super(code: 'TIMEOUT');
}

import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:logger/logger.dart';

/// Realtime event from SSE
class RealtimeEvent {

  const RealtimeEvent({required this.type, required this.data});

  final String type;
  final String data;

  Map<String, dynamic>? get jsonData {
    try {
      return jsonDecode(data) as Map<String, dynamic>;
    } catch (_) {
      return null;
    }
  }

  @override
  String toString() => 'RealtimeEvent(type: $type, data: $data)';
}

/// Realtime client using Server-Sent Events (SSE)
class RealtimeClient {

  RealtimeClient({
    required this.baseUrl,
    bool enableLogging = true,
  }) : _logger = Logger(
          level: enableLogging ? Level.debug : Level.off,
          printer: PrettyPrinter(
            methodCount: 0,
            errorMethodCount: 5,
            lineLength: 80,
            colors: true,
            printEmojis: true,
          ),
        );
  final String baseUrl;
  final Logger _logger;
  String? _accessToken;

  http.Client? _client;
  StreamController<RealtimeEvent>? _controller;
  StreamSubscription<String>? _subscription;
  String? _currentEventType;
  final List<String> _currentDataLines = [];
  bool _isClosed = false;

  /// Set access token for authenticated connections
  void setAccessToken(String? token) {
    _accessToken = token;
  }

  /// Listen to realtime events
  /// 
  /// Example:
  /// ```dart
  /// client.realtime.listen().listen((event) {
  ///   print('Event: ${event.type}');
  ///   print('Data: ${event.jsonData}');
  /// });
  /// ```
  Stream<RealtimeEvent> listen() {
    _isClosed = false;
    _controller = StreamController<RealtimeEvent>.broadcast(onCancel: close);

    _startListening();

    return _controller!.stream;
  }

  Future<void> _startListening() async {
    _client = http.Client();

    final uri = Uri.parse('$baseUrl/events');
    final request = http.Request('GET', uri)
      ..headers.addAll({
        'Accept': 'text/event-stream',
        'Cache-Control': 'no-cache',
      });

    if (_accessToken != null && _accessToken!.isNotEmpty) {
      request.headers['Authorization'] = 'Bearer $_accessToken';
    }

    try {
      _logger.d('→ Connecting to SSE: $uri');
      final response = await _client!.send(request);

      if (response.statusCode < 200 || response.statusCode >= 300) {
        throw http.ClientException(
          'SSE connection failed with status ${response.statusCode}',
          request.url,
        );
      }

      _logger.i('← SSE connected');

      _subscription = utf8.decoder
          .bind(response.stream)
          .transform(const LineSplitter())
          .listen(
            _handleLine,
            onError: _handleError,
            onDone: _flushAndClose,
            cancelOnError: true,
          );
    } catch (e) {
      _handleError(e);
    }
  }

  void _handleLine(String line) {
    if (line.startsWith('event:')) {
      _currentEventType = line.substring(6).trim();
      return;
    }

    if (line.startsWith('data:')) {
      _currentDataLines.add(line.substring(5).trim());
      return;
    }

    if (line.isEmpty) {
      _flushEvent();
    }
  }

  void _flushEvent() {
    if (_currentDataLines.isEmpty) {
      _currentEventType = null;
      return;
    }

    final event = RealtimeEvent(
      type: _currentEventType ?? 'message',
      data: _currentDataLines.join('\n'),
    );

    _logger.d('← Event: ${event.type}');
    _controller?.add(event);

    _currentEventType = null;
    _currentDataLines.clear();
  }

  void _handleError(Object error) {
    _logger.e('SSE connection error', error: error);
    _controller?.addError(error);
  }

  Future<void> _flushAndClose() async {
    _flushEvent();
    await close();
  }

  /// Close the realtime connection
  Future<void> close() async {
    if (_isClosed) {
      return;
    }
    _isClosed = true;

    _logger.d('Closing SSE connection');

    await _subscription?.cancel();
    _subscription = null;
    _client?.close();
    _client = null;
    await _controller?.close();
    _controller = null;
  }
}

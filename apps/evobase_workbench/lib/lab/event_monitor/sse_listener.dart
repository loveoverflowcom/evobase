import 'dart:async';
import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:logger/logger.dart';

/// SSE (Server-Sent Events) listener for /events endpoint.
/// Used only in lab for debugging.
class SseListener {
  final String baseUrl;
  final String? accessToken;
  final Logger _logger = Logger();

  http.Client? _client;
  StreamController<SseEvent>? _controller;
  StreamSubscription<String>? _subscription;
  String? _currentEventType;
  final List<String> _currentDataLines = [];
  bool _isClosed = false;

  SseListener({required this.baseUrl, this.accessToken});

  Stream<SseEvent> listen() {
    _isClosed = false;
    _controller = StreamController<SseEvent>.broadcast(onCancel: close);

    _startListening();

    return _controller!.stream;
  }

  Future<void> _startListening() async {
    _client = http.Client();

    final request = http.Request('GET', Uri.parse('$baseUrl/events'))
      ..headers.addAll({
        'Accept': 'text/event-stream',
        'Cache-Control': 'no-cache',
      });

    if (accessToken != null) {
      request.headers['Authorization'] = 'Bearer $accessToken';
    }

    try {
      final response = await _client!.send(request);
      if (response.statusCode < 200 || response.statusCode >= 300) {
        throw http.ClientException(
          'SSE request failed with status code ${response.statusCode}',
          request.url,
        );
      }

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

    _controller?.add(
      SseEvent(
        type: _currentEventType ?? 'message',
        data: _currentDataLines.join('\n'),
      ),
    );
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

  Future<void> close() async {
    if (_isClosed) {
      return;
    }
    _isClosed = true;

    await _subscription?.cancel();
    _subscription = null;
    _client?.close();
    _client = null;
    await _controller?.close();
    _controller = null;
  }
}

class SseEvent {
  final String type;
  final String data;

  SseEvent({required this.type, required this.data});

  Map<String, dynamic>? get jsonData {
    try {
      return jsonDecode(data) as Map<String, dynamic>;
    } catch (_) {
      return null;
    }
  }

  @override
  String toString() => 'SseEvent(type: $type, data: $data)';
}

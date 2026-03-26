import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:http/http.dart' as http;

import '../../lab/event_monitor/sse_listener.dart';

class MessagingLogEntry {
  final DateTime timestamp;
  final String event;
  final String data;
  final bool outbound;

  const MessagingLogEntry({
    required this.timestamp,
    required this.event,
    required this.data,
    required this.outbound,
  });
}

class MessagingController extends ChangeNotifier {
  final String baseUrl;
  final http.Client _httpClient = http.Client();

  SseListener? _sseListener;
  StreamSubscription<SseEvent>? _sseSubscription;

  final List<MessagingLogEntry> messages = [];
  bool isSubscribing = false;
  bool isSending = false;
  String? activeTopic;
  String? errorMessage;

  MessagingController({required this.baseUrl});

  Future<void> subscribe(String topic) async {
    final trimmedTopic = topic.trim();
    if (trimmedTopic.isEmpty) {
      errorMessage = 'Topic is required';
      notifyListeners();
      return;
    }

    await _closeSubscription();
    isSubscribing = true;
    errorMessage = null;
    activeTopic = trimmedTopic;
    notifyListeners();

    _sseListener = SseListener(
      baseUrl: baseUrl,
      path: '/evobase_messaging/subscribe',
      queryParameters: {'topic': trimmedTopic},
    );

    _sseSubscription = _sseListener!.listen().listen(
      (event) {
        _appendMessage(
          MessagingLogEntry(
            timestamp: DateTime.now(),
            event: event.type,
            data: event.data,
            outbound: false,
          ),
        );
      },
      onError: (Object error) {
        errorMessage = error.toString();
        notifyListeners();
      },
      onDone: () {
        isSubscribing = false;
        notifyListeners();
      },
    );
  }

  Future<void> send({
    required String topic,
    required String toUserId,
    required String event,
    required String payloadJson,
  }) async {
    final trimmedTopic = topic.trim();
    final trimmedToUserId = toUserId.trim();
    final resolvedTopic = trimmedTopic.isNotEmpty
        ? trimmedTopic
        : (trimmedToUserId.isNotEmpty ? 'user:$trimmedToUserId' : '');
    final resolvedEvent = event.trim().isEmpty ? 'message' : event.trim();

    if (resolvedTopic.isEmpty && trimmedToUserId.isEmpty) {
      errorMessage = 'Provide topic or to_user_id';
      notifyListeners();
      return;
    }

    final dynamic payload;
    try {
      payload = jsonDecode(payloadJson);
    } catch (_) {
      errorMessage = 'Payload must be valid JSON';
      notifyListeners();
      return;
    }

    isSending = true;
    errorMessage = null;
    notifyListeners();

    final uri = Uri.parse(baseUrl).resolve('evobase_messaging/publish');
    final body = <String, dynamic>{
      'event': resolvedEvent,
      'payload': payload,
    };
    if (trimmedToUserId.isNotEmpty) {
      body['to_user_id'] = trimmedToUserId;
    } else {
      body['topic'] = resolvedTopic;
    }

    try {
      final response = await _httpClient.post(
        uri,
        headers: {'Content-Type': 'application/json', 'Accept': 'application/json'},
        body: jsonEncode(body),
      );
      if (response.statusCode < 200 || response.statusCode >= 300) {
        errorMessage =
            'Publish failed (${response.statusCode}): ${response.body}';
      } else {
        _appendMessage(
          MessagingLogEntry(
            timestamp: DateTime.now(),
            event: resolvedEvent,
            data: jsonEncode(payload),
            outbound: true,
          ),
        );
      }
    } catch (error) {
      errorMessage = error.toString();
    } finally {
      isSending = false;
      notifyListeners();
    }
  }

  Future<void> _closeSubscription() async {
    await _sseSubscription?.cancel();
    _sseSubscription = null;
    await _sseListener?.close();
    _sseListener = null;
    isSubscribing = false;
  }

  void _appendMessage(MessagingLogEntry entry) {
    messages.insert(0, entry);
    if (messages.length > 300) {
      messages.removeRange(300, messages.length);
    }
    notifyListeners();
  }

  @override
  void dispose() {
    _sseSubscription?.cancel();
    _sseListener?.close();
    _httpClient.close();
    super.dispose();
  }
}

import 'package:flutter/material.dart';

import '../../config.dart' show AppConfig;
import 'messaging_controller.dart';

class MessagingPage extends StatefulWidget {
  const MessagingPage({super.key});

  @override
  State<MessagingPage> createState() => _MessagingPageState();
}

class _MessagingPageState extends State<MessagingPage> {
  late final MessagingController _controller;
  final TextEditingController _topicController = TextEditingController();
  final TextEditingController _toUserIdController = TextEditingController();
  final TextEditingController _eventController = TextEditingController(
    text: 'message',
  );
  final TextEditingController _payloadController = TextEditingController(
    text: '{\n  "text": "hello from evobase_messaging"\n}',
  );

  @override
  void initState() {
    super.initState();
    _controller = MessagingController(baseUrl: AppConfig.current.baseUrl)
      ..addListener(_onControllerChanged);
  }

  @override
  void dispose() {
    _controller.removeListener(_onControllerChanged);
    _controller.dispose();
    _topicController.dispose();
    _toUserIdController.dispose();
    _eventController.dispose();
    _payloadController.dispose();
    super.dispose();
  }

  void _onControllerChanged() {
    if (mounted) {
      setState(() {});
    }
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        children: [
          Card(
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _topicController,
                      decoration: const InputDecoration(
                        labelText: 'Topic',
                        hintText: 'example: notifications.global',
                        border: OutlineInputBorder(),
                      ),
                    ),
                  ),
                  const SizedBox(width: 12),
                  FilledButton(
                    onPressed: _controller.isSubscribing
                        ? null
                        : () => _controller.subscribe(_topicController.text),
                    child: const Text('Subscribe'),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 12),
          if (_controller.activeTopic != null)
            Align(
              alignment: Alignment.centerLeft,
              child: Text('Subscribed topic: ${_controller.activeTopic}'),
            ),
          if (_controller.errorMessage != null)
            Align(
              alignment: Alignment.centerLeft,
              child: Text(
                _controller.errorMessage!,
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ),
          const SizedBox(height: 8),
          Expanded(
            child: Card(
              child: _controller.messages.isEmpty
                  ? const Center(child: Text('No messages yet'))
                  : ListView.separated(
                      reverse: false,
                      padding: const EdgeInsets.all(12),
                      itemCount: _controller.messages.length,
                      separatorBuilder: (_, _) => const Divider(height: 16),
                      itemBuilder: (context, index) {
                        final item = _controller.messages[index];
                        return Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              '${item.outbound ? 'OUT' : 'IN'} • ${item.event} • ${item.timestamp.toIso8601String()}',
                              style: Theme.of(context).textTheme.labelMedium,
                            ),
                            const SizedBox(height: 4),
                            SelectableText(item.data),
                          ],
                        );
                      },
                    ),
            ),
          ),
          const SizedBox(height: 12),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Column(
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: TextField(
                          controller: _toUserIdController,
                          decoration: const InputDecoration(
                            labelText: 'to_user_id (optional)',
                            border: OutlineInputBorder(),
                          ),
                        ),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: TextField(
                          controller: _eventController,
                          decoration: const InputDecoration(
                            labelText: 'event',
                            border: OutlineInputBorder(),
                          ),
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  TextField(
                    controller: _payloadController,
                    minLines: 6,
                    maxLines: 10,
                    decoration: const InputDecoration(
                      labelText: 'JSON payload',
                      border: OutlineInputBorder(),
                    ),
                  ),
                  const SizedBox(height: 12),
                  Align(
                    alignment: Alignment.centerRight,
                    child: FilledButton(
                      onPressed: _controller.isSending
                          ? null
                          : () => _controller.send(
                              topic: _topicController.text,
                              toUserId: _toUserIdController.text,
                              event: _eventController.text,
                              payloadJson: _payloadController.text,
                            ),
                      child: const Text('Send'),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

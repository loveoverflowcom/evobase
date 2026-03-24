// ignore_for_file: avoid_print

import 'package:evobase_flutter/evobase.dart';

void main() async {
  // Initialize client
  final client = await EvobaseClient.create(
    baseUrl: 'http://localhost:3000',
  );

  // Example 1: Register
  print('=== Register ===');
  final registerResult = await client.auth
      .register(
        username: 'demo_user',
        password: 'demo_password',
      )
      .run();

  registerResult.match(
    (error) => print('Register error: ${error.message}'),
    (response) => print('Registered as ${response.username}'),
  );

  // Example 2: Login
  print('\n=== Login ===');
  final loginResult = await client.auth
      .login(
        username: 'demo_user',
        password: 'demo_password',
      )
      .run();

  loginResult.match(
    (error) => print('Login error: ${error.message}'),
    (response) => print('Logged in as ${response.username}'),
  );

  // Example 3: Query data
  print('\n=== Query Users ===');
  final queryResult = await client.rest
      .query(
        'users',
        params: QueryParams(
          select: ['id', 'username', 'created_at'],
          limit: 5,
        ),
      )
      .run();

  queryResult.match(
    (error) => print('Query error: ${error.message}'),
    (users) {
      print('Found ${users.length} users:');
      for (final user in users) {
        print('  - ${user['username']} (${user['id']})');
      }
    },
  );

  // Example 4: Insert data
  print('\n=== Insert Post ===');
  final insertResult = await client.rest
      .insert(
        'posts',
        InsertBody(
          records: [
            {
              'title': 'Hello Evobase',
              'content': 'This is my first post!',
              'author_id': client.auth.getUserId(),
            }
          ],
          returning: ['id', 'title'],
        ),
      )
      .run();

  insertResult.match(
    (error) => print('Insert error: ${error.message}'),
    (posts) => print('Created post: ${posts.first['title']}'),
  );

  // Example 5: Update data
  print('\n=== Update Post ===');
  final updateResult = await client.rest
      .update(
        'posts',
        UpdateBody(
          set: {'content': 'Updated content!'},
          returning: ['id', 'content'],
        ),
        params: QueryParams(
          filter: {'author_id.eq': client.auth.getUserId()},
        ),
      )
      .run();

  updateResult.match(
    (error) => print('Update error: ${error.message}'),
    (posts) => print('Updated ${posts.length} posts'),
  );

  // Example 6: Realtime events
  print('\n=== Listening to Realtime Events ===');
  print('(Press Ctrl+C to stop)');

  client.realtime.listen().listen(
    (event) {
      print('Event: ${event.type}');
      print('Data: ${event.jsonData}');
    },
    onError: (error) => print('Realtime error: $error'),
  );

  // Keep the program running
  await Future.delayed(const Duration(seconds: 30));

  // Cleanup
  print('\n=== Cleanup ===');
  await client.realtime.close();
  client.close();
  print('Done!');
}

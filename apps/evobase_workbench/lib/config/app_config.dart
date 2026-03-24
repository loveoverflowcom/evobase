/// Application configuration
class AppConfig {
  final String baseUrl;
  final String environment;
  final Duration connectTimeout;
  final Duration receiveTimeout;

  const AppConfig({
    required this.baseUrl,
    required this.environment,
    this.connectTimeout = const Duration(seconds: 30),
    this.receiveTimeout = const Duration(seconds: 30),
  });

  static const dev = AppConfig(
    baseUrl: 'http://localhost:3000',
    environment: 'development',
  );

  static const prod = AppConfig(
    baseUrl: 'https://api.evobase.com',
    environment: 'production',
  );

  // Default to dev
  static AppConfig current = dev;

  static void setEnvironment(AppConfig config) {
    current = config;
  }
}

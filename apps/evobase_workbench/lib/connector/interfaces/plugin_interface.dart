/// Abstract interface for plugins
/// Future extensibility for custom integrations
abstract class PluginInterface {
  String get name;
  String get version;
  Future<void> initialize();
  Future<void> dispose();
  Future<dynamic> execute(String action, Map<String, dynamic> params);
}

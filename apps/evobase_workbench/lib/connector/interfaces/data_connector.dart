/// Abstract interface for data connectors
/// Allows pluggable implementations for different data sources
abstract class DataConnector {
  Future<void> connect();
  Future<void> disconnect();
  Future<bool> isConnected();
  Future<Map<String, dynamic>> query(String query);
  Future<void> execute(String command);
}

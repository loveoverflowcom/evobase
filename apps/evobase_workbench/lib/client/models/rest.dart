import 'package:json_annotation/json_annotation.dart';

part 'rest.g.dart';

/// Mirror of evobase-protocol TableQueryParams
@JsonSerializable()
class TableQueryParams {
  final String? select;
  final String? order;
  final int? limit;
  final int? offset;

  TableQueryParams({this.select, this.order, this.limit, this.offset});

  factory TableQueryParams.fromJson(Map<String, dynamic> json) =>
      _$TableQueryParamsFromJson(json);

  Map<String, dynamic> toJson() => _$TableQueryParamsToJson(this);

  Map<String, String> toQueryMap() {
    final map = <String, String>{};
    if (select != null) map['select'] = select!;
    if (order != null) map['order'] = order!;
    if (limit != null) map['limit'] = limit.toString();
    if (offset != null) map['offset'] = offset.toString();
    return map;
  }
}

/// Mirror of evobase-protocol InsertBody
/// Can be single object or array of objects
class InsertBody {
  final dynamic data;

  InsertBody.single(Map<String, dynamic> object) : data = object;
  InsertBody.multiple(List<Map<String, dynamic>> objects) : data = objects;

  Map<String, dynamic>? get asSingle =>
      data is Map<String, dynamic> ? data : null;

  List<Map<String, dynamic>>? get asMultiple =>
      data is List ? List<Map<String, dynamic>>.from(data) : null;

  dynamic toJson() => data;
}

/// Mirror of evobase-protocol PatchBody
class PatchBody {
  final Map<String, dynamic> fields;

  PatchBody(this.fields);

  Map<String, dynamic> toJson() => fields;
}

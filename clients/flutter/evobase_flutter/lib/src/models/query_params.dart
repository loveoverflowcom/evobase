/// Query parameters for REST API
class QueryParams {

  QueryParams({
    this.select,
    this.filter,
    this.order,
    this.limit,
    this.offset,
  });
  final List<String>? select;
  final Map<String, dynamic>? filter;
  final List<String>? order;
  final int? limit;
  final int? offset;

  Map<String, dynamic> toQueryMap() {
    final map = <String, dynamic>{};

    if (select != null && select!.isNotEmpty) {
      map['select'] = select!.join(',');
    }

    if (filter != null && filter!.isNotEmpty) {
      for (final entry in filter!.entries) {
        map[entry.key] = entry.value;
      }
    }

    if (order != null && order!.isNotEmpty) {
      map['order'] = order!.join(',');
    }

    if (limit != null) {
      map['limit'] = limit.toString();
    }

    if (offset != null) {
      map['offset'] = offset.toString();
    }

    return map;
  }
}

/// Insert body for REST API
class InsertBody {

  InsertBody({
    required this.records,
    this.returning,
  });
  final List<Map<String, dynamic>> records;
  final List<String>? returning;

  Map<String, dynamic> toJson() {
    final json = <String, dynamic>{
      'records': records,
    };

    if (returning != null && returning!.isNotEmpty) {
      json['returning'] = returning;
    }

    return json;
  }
}

/// Update body for REST API
class UpdateBody {

  UpdateBody({
    required this.set,
    this.returning,
  });
  final Map<String, dynamic> set;
  final List<String>? returning;

  Map<String, dynamic> toJson() {
    final json = <String, dynamic>{
      'set': set,
    };

    if (returning != null && returning!.isNotEmpty) {
      json['returning'] = returning;
    }

    return json;
  }
}

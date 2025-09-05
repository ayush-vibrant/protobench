@0xf1234567890abcde;

struct MetricPoint {
  timestamp @0 :Int64;
  hostname @1 :Text;
  cpuPercent @2 :Float32;
  memoryBytes @3 :UInt64;
  diskIoOps @4 :UInt32;
  tags @5 :List(Tag);
  
  struct Tag {
    key @0 :Text;
    value @1 :Text;
  }
}

struct MetricQuery {
  startTime @0 :Int64;
  endTime @1 :Int64;
  hostnameFilter @2 :Text;
}

struct MetricStatistics {
  count @0 :UInt64;
  avgCpuPercent @1 :Float32;
  avgMemoryBytes @2 :UInt64;
  avgDiskIoOps @3 :Float32;
  timeRangeSeconds @4 :Int64;
}

interface MetricsService {
  submitMetric @0 (metric :MetricPoint) -> ();
  queryMetrics @1 (query :MetricQuery) -> (metrics :List(MetricPoint));
  getStatistics @2 (query :MetricQuery) -> (statistics :MetricStatistics);
}
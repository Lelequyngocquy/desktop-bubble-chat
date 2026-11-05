export interface ProcessMetricItem {
  pid: number;
  name: string;
  category: "host" | "dashboard" | "messenger" | "gpu" | "utility" | string;
  accountIndex?: number | null;
  accountName?: string | null;
  workingSetMb: number;
  cpuPercent: number;
  isSuspended: boolean;
}

export interface SystemPerformanceSnapshot {
  timestamp: number;
  totalRamMb: number;
  totalCpuPercent: number;
  hostRamMb: number;
  dashboardRamMb: number;
  activeAccountsCount: number;
  suspendedAccountsCount: number;
  estimatedSavedRamMb: number;
  cpuCores: number;
  items: ProcessMetricItem[];
}

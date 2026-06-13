export interface BadgeInfo {
  cls: string
  label: string
}

/**
 * Map article processing status to badge CSS class and display label.
 * Covers 7 scenarios: 4 explicit statuses + 2 legacy fallbacks + unknown.
 *
 * @param status  New status field ('pending' | 'processing' | 'matched' | 'skipped')
 * @param processedAt  Legacy processed_at field for backwards compatibility
 * @returns Badge information for rendering
 */
export function articleStatusBadge(status?: string, processedAt?: string | null): BadgeInfo {
  // New explicit status takes priority
  if (status) {
    switch (status) {
      case 'matched':
        return { cls: 'badge-success', label: '已匹配' }
      case 'pending':
        return { cls: 'badge-warn', label: '待处理' }
      case 'processing':
        return { cls: 'badge-info', label: '处理中' }
      case 'skipped':
        return { cls: 'badge-muted', label: '已跳过' }
      default:
        return { cls: 'badge-warn', label: '未知' }
    }
  }

  // Legacy fallback: use processed_at to infer status
  if (processedAt != null) {
    return { cls: 'badge-success', label: '已处理' }
  }
  return { cls: 'badge-warn', label: '待处理' }
}

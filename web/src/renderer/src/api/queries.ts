import client from './client'

export interface HotEvent {
  id: number
  keyword_id: number
  hour_bucket: string
  count: number
  mean_historical: number
  stddev_historical: number
  created_at: string
}

export interface Article {
  id: number
  source_id: number
  link: string
  title: string
  summary: string
  content: string
  published_at: string | null
  fetched_at: string
  processed_at: string | null
  /** 'pending' | 'processing' | 'matched' | 'skipped' — may be absent in legacy data */
  status?: string
}

export interface Source {
  id: number
  type: string
  name: string
  url: string
  config: string
  enabled: boolean
  interval_seconds: number
  article_count: number
  last_fetched_at: string | null
  created_at: string
  updated_at: string
}

export interface Keyword {
  id: number
  word: string
  case_sensitive: boolean
  enabled: boolean
  std_multiplier: number
  min_hot_count: number
  created_at: string
}

export interface TrendPoint {
  hour_bucket: string
  count: number
}

export interface TrendResponse {
  keyword_id: number
  keyword: string
  points: TrendPoint[]
}

export interface PushRecord {
  id: number
  hot_event_id: number
  channel_id: number
  channel_name: string
  /** 'pending' | 'failed' | 'sent' | 'dead' */
  status: string
  retry_count: number
  next_retry_at: string | null
  last_error: string | null
  created_at: string
  updated_at: string
}

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  per_page: number
}

export const queryApi = {
  getHotspots: (params?: { page?: number; per_page?: number; keyword_id?: number }) =>
    client.get<{ data: PaginatedResponse<HotEvent> }>('/hotspots', { params }).then((r) => r.data.data),

  getArticles: (params?: { page?: number; per_page?: number; source_id?: number; processed?: boolean; status?: string }) =>
    client.get<{ data: PaginatedResponse<Article> }>('/articles', { params }).then((r) => r.data.data),

  getSources: () =>
    client.get<{ data: Source[] }>('/sources').then((r) => r.data.data),

  getKeywords: () =>
    client.get<{ data: Keyword[] }>('/keywords').then((r) => r.data.data),

  getTrend: (keywordId: number, hours?: number) =>
    client.get<{ data: TrendResponse }>(`/trend/${keywordId}`, { params: hours ? { hours } : undefined }).then((r) => r.data.data),

  getPushRecords: (hotspotId: number) =>
    client.get<{ data: PushRecord[] }>(`/hotspots/${hotspotId}/push-records`).then((r) => r.data.data),

  triggerFilter: () => client.post('/trigger/filter').then((r) => r.data),

  triggerPusher: () => client.post('/trigger/pusher').then((r) => r.data)
}

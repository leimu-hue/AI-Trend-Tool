import client from './client'

export interface HotEvent {
  id: number
  keyword_id: number
  keyword: string
  current_count: number
  mean: number
  stddev: number
  std_multiplier: number
  hour_bucket: string
  status: string
  created_at: string
}

export interface Article {
  id: number
  source_id: number
  source_name: string
  title: string
  link: string
  matched_keywords: string | null
  published_at: string | null
  processed_at: string | null
  created_at: string
}

export interface DashboardStats {
  source_count: number
  keyword_count: number
  today_articles: number
  active_hotspots: number
}

export const queryApi = {
  getHotspots: (params?: { limit?: number; offset?: number }) =>
    client.get<HotEvent[]>('/hotspots', { params }).then((r) => r.data),

  getArticles: (params?: { limit?: number; offset?: number; source_id?: number }) =>
    client.get<Article[]>('/articles', { params }).then((r) => r.data),

  getStats: () => client.get<DashboardStats>('/stats').then((r) => r.data),

  triggerFilter: () => client.post('/filter/run').then((r) => r.data)
}

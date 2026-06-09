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

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  per_page: number
}

export const queryApi = {
  getHotspots: (params?: { page?: number; per_page?: number; keyword_id?: number }) =>
    client.get<{ data: PaginatedResponse<HotEvent> }>('/hotspots', { params }).then((r) => r.data.data),

  getArticles: (params?: { page?: number; per_page?: number; source_id?: number; processed?: boolean }) =>
    client.get<{ data: PaginatedResponse<Article> }>('/articles', { params }).then((r) => r.data.data),

  triggerFilter: () => client.post('/trigger/filter').then((r) => r.data)
}

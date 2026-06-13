import client from './client'

export interface PushChannel {
  id: number
  name: string
  channel_type: string
  // 后端 config 为 JSON 字符串，list 返回字符串，需前端 JSON.parse
  config: string
  enabled: boolean
  push_count: number | null
  last_pushed_at: string | null
  created_at: string
  updated_at: string
}

export interface CreateChannelRequest {
  name: string
  channel_type?: string
  // 后端期望 JSON 字符串
  config: string
}

export interface UpdateChannelRequest {
  name?: string
  channel_type?: string
  config?: string
  enabled?: boolean
}

export const channelApi = {
  list: () => client.get<{ data: PushChannel[] }>('/channels').then((r) => r.data.data),

  get: (id: number) => client.get<{ data: PushChannel }>(`/channels/${id}`).then((r) => r.data.data),

  create: (data: CreateChannelRequest) =>
    client.post<{ data: PushChannel }>('/channels', data).then((r) => r.data.data),

  update: (id: number, data: UpdateChannelRequest) =>
    client.post<{ data: PushChannel }>(`/channels/${id}/update`, data).then((r) => r.data.data),

  delete: (id: number) => client.post(`/channels/${id}/delete`).then((r) => r.data),

  test: (id: number) => client.post(`/channels/${id}/test`).then((r) => r.data)
}

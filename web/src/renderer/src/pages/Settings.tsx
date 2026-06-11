import { useState, useEffect } from 'react'
import client from '../api/client'

interface SettingsData {
  parser?: {
    max_concurrent_fetches?: number
    default_timeout_seconds?: number
    interval_seconds?: number
  }
  filter?: {
    batch_size?: number
    interval_seconds?: number
    history_hours?: number
    min_history_hours?: number
  }
  pusher?: {
    interval_seconds?: number
    max_retries?: number
    retry_base_seconds?: number
  }
  server?: {
    host?: string
    port?: number
  }
}

const DEFAULTS: SettingsData = {
  parser: {
    max_concurrent_fetches: 5,
    default_timeout_seconds: 30,
    interval_seconds: 30
  },
  filter: {
    batch_size: 100,
    interval_seconds: 300,
    history_hours: 24,
    min_history_hours: 4
  },
  pusher: {
    interval_seconds: 10,
    max_retries: 3,
    retry_base_seconds: 60
  },
  server: {
    host: '0.0.0.0',
    port: 3000
  }
}

const GROUP_LABELS: Record<string, string> = {
  parser: '解析器配置',
  filter: '过滤器配置',
  pusher: '推送器配置',
  server: '服务器配置'
}

const FIELD_LABELS: Record<string, Record<string, string>> = {
  parser: {
    max_concurrent_fetches: '最大并发抓取数',
    default_timeout_seconds: '默认超时时间（秒）',
    interval_seconds: '默认抓取间隔（秒）'
  },
  filter: {
    batch_size: '批处理大小',
    interval_seconds: '运行间隔（秒）',
    history_hours: '历史窗口（小时）',
    min_history_hours: '最小历史数据（小时）'
  },
  pusher: {
    interval_seconds: '轮询间隔（秒）',
    max_retries: '最大重试次数',
    retry_base_seconds: '重试基础间隔（秒）'
  },
  server: {
    host: '监听地址',
    port: '端口'
  }
}

export default function Settings() {
  const [data, setData] = useState<SettingsData | null>(null)
  const [loading, setLoading] = useState(true)
  const [usingDefaults, setUsingDefaults] = useState(false)

  useEffect(() => {
    let cancelled = false
    client
      .get<{ data: SettingsData }>('/settings')
      .then((r) => {
        if (!cancelled) {
          setData(r.data.data)
          setUsingDefaults(false)
        }
      })
      .catch(() => {
        if (!cancelled) {
          setData(DEFAULTS)
          setUsingDefaults(true)
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })
    return () => {
      cancelled = true
    }
  }, [])

  if (loading) {
    return <p style={{ color: 'var(--color-muted)', padding: 16 }}>加载中...</p>
  }

  if (!data) return null

  const groups = Object.keys(GROUP_LABELS) as (keyof SettingsData)[]

  return (
    <div>
      {usingDefaults && (
        <p
          style={{
            color: 'var(--color-muted)',
            fontSize: 12,
            marginBottom: 16,
            marginTop: 0
          }}
        >
          默认配置
        </p>
      )}
      <div className="settings-grid">
        {groups.map((group) => {
          const cfg = data[group] as Record<string, number | string>
          if (!cfg) return null
          const labels = FIELD_LABELS[group]
          return (
            <div key={group} className="settings-group">
              <h3>{GROUP_LABELS[group]}</h3>
              {Object.entries(labels).map(([key, label]) => (
                <div key={key} className="setting-row">
                  <span className="setting-label">{label}</span>
                  <span className="setting-value">{cfg[key]}</span>
                </div>
              ))}
            </div>
          )
        })}
      </div>
    </div>
  )
}

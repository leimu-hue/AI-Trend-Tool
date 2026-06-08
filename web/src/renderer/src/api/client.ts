import axios from 'axios'
import { notifyError } from '../lib/notification'

/**
 * axios 拦截器运行在 React 树之外，无法使用 hooks。
 * 通过 lib/notification 的统一服务发送错误通知：
 * 优先使用已挂载的上下文感知 API，无挂载时回退至 antd 静态单例。
 * 详见 lib/notification.ts 中的 D7 双实例说明。
 */

const client = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api/v1',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json'
  }
})

client.interceptors.request.use((config) => {
  const token = localStorage.getItem('api_token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

client.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response) {
      if (error.response.status === 401) {
        localStorage.removeItem('api_token')
        window.location.hash = '#/auth'
        return Promise.reject(error)
      }
      const msg = error.response.data?.error?.message || error.response.data?.message
      if (msg) {
        notifyError(msg)
      } else {
        notifyError(`服务器错误: ${error.response.status}`)
      }
    } else {
      notifyError('网络错误，请检查后端服务是否启动')
    }
    return Promise.reject(error)
  }
)

export default client

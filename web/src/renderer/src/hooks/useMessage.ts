import { App } from 'antd'
import { useEffect } from 'react'
import { setNotificationApi } from '../lib/notification'

/**
 * 通知 hook —— 通过 App.useApp() 获取上下文感知 notification API，
 * 并将其注入 lib/notification 服务，使树外拦截器也能使用同一实例。
 *
 * 挂载位置：App.tsx 顶层（确保在所有子组件之前初始化）
 */
export function useNotificationBridge() {
  const { notification } = App.useApp()

  useEffect(() => {
    setNotificationApi(notification)
  }, [notification])

  return notification
}

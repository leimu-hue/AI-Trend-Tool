/**
 * 统一通知服务 —— 桥接 React 树内/树外两种通知场景
 *
 * 双实例说明 (D7):
 * ─────────────────
 * • 树内 (React components)：通过 useNotify() hook 获取 App.useApp() 上下文
 *   感知的 notification API，继承 ConfigProvider 的主题、国际化等配置。
 *
 * • 树外 (axios 拦截器等非 React 环境)：通过 notifyError() 使用模块缓存的
 *   上下文 API；若尚未挂载则回退至 antd 静态单例 notification.error()。
 *
 * 两者共享相同的 placement / duration / closeIcon 配置。
 * antd 5 中静态 notification 也是全局单例，不会产生 DOM 冲突，
 * 唯一差异是静态实例不继承 ConfigProvider 主题（在 dark 模式下视觉一致，
 * 因 ThemeProvider 已全局应用暗色算法）。
 */
import { notification as staticNotification } from 'antd'
import { App } from 'antd'
import type { NotificationInstance } from 'antd/es/notification/interface'

/** 共享通知配置 */
const PLACEMENT = 'bottomRight' as const
const DURATION_ERROR = 3
const DURATION_DEFAULT = 2

/**
 * 模块级缓存的上下文感知 notification API。
 * 由 NotificationProvider 在 React 挂载时通过 setNotificationApi() 写入。
 */
let contextApi: NotificationInstance | null = null

export function setNotificationApi(api: NotificationInstance): void {
  contextApi = api
}

/**
 * 树外错误通知（供 axios 拦截器等非 React 环境使用）
 * 优先使用缓存的上下文 API，保证主题一致性；无上下文时回退静态单例。
 */
export function notifyError(msg: string): void {
  const api = contextApi ?? staticNotification
  api.error({
    message: msg,
    placement: PLACEMENT,
    duration: DURATION_ERROR,
    closeIcon: false
  })
}

/**
 * 树内通知 hook（供 React 组件使用）
 * 始终使用 App.useApp() 上下文感知实例，确保继承 ConfigProvider 配置。
 */
export function useNotify() {
  const { notification } = App.useApp()

  return {
    notifySuccess: (msg: string) =>
      notification.success({
        message: msg,
        placement: PLACEMENT,
        duration: DURATION_DEFAULT,
        closeIcon: false
      }),
    notifyError: (msg: string) =>
      notification.error({
        message: msg,
        placement: PLACEMENT,
        duration: DURATION_ERROR,
        closeIcon: false
      }),
    notifyInfo: (msg: string) =>
      notification.info({
        message: msg,
        placement: PLACEMENT,
        duration: DURATION_DEFAULT,
        closeIcon: false
      })
  }
}

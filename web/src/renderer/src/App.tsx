import { Routes, Route, Navigate } from 'react-router-dom'
import { useNotificationBridge } from './hooks/useMessage'
import ProtectedRoute from './components/ProtectedRoute'
import Layout from './components/Layout'
import AuthPage from './pages/Auth'
import Dashboard from './pages/Dashboard'
import Sources from './pages/Sources'
import Keywords from './pages/Keywords'
import Channels from './pages/Channels'
import Articles from './pages/Articles'
import Settings from './pages/Settings'
import Tokens from './pages/Tokens'

export default function App() {
  // 将上下文感知 notification API 注入 lib/notification 服务，
  // 使 axios 拦截器等树外代码也能使用同一实例
  useNotificationBridge()

  return (
    <Routes>
      <Route path="/auth" element={<AuthPage />} />
      <Route element={<ProtectedRoute />}>
        <Route element={<Layout />}>
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/sources" element={<Sources />} />
          <Route path="/keywords" element={<Keywords />} />
          <Route path="/channels" element={<Channels />} />
          <Route path="/articles" element={<Articles />} />
          <Route path="/tokens" element={<Tokens />} />
          <Route path="/settings" element={<Settings />} />
        </Route>
      </Route>
      <Route path="*" element={<Navigate to="/dashboard" replace />} />
    </Routes>
  )
}

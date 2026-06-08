import { ConfigProvider, theme, App } from 'antd'
import type { ReactNode } from 'react'
import { themeTokens } from './tokens'

export function ThemeProvider({ children }: { children: ReactNode }) {
  return (
    <ConfigProvider
      theme={{
        ...themeTokens,
        algorithm: theme.darkAlgorithm
      }}
    >
      <App>{children}</App>
    </ConfigProvider>
  )
}

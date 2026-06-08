import { Spin } from 'antd'

interface LoadingProps {
  fullPage?: boolean
}

export default function Loading({ fullPage = false }: LoadingProps) {
  return (
    <div className={`flex items-center justify-center ${fullPage ? 'min-h-screen' : 'min-h-[200px]'}`}>
      <Spin tip="加载中..." />
    </div>
  )
}

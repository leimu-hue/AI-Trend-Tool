import { Empty as AntEmpty, Button } from 'antd'

interface EmptyProps {
  description?: string
  actionText?: string
  onAction?: () => void
}

export default function Empty({ description = '暂无数据', actionText, onAction }: EmptyProps) {
  return (
    <div className="flex items-center justify-center min-h-[200px]">
      <AntEmpty description={description}>
        {actionText && onAction && (
          <Button type="primary" onClick={onAction}>
            {actionText}
          </Button>
        )}
      </AntEmpty>
    </div>
  )
}

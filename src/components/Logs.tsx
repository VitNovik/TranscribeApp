import { X, Copy, Download, Trash2, ArrowDown } from 'lucide-react';
import { useLogs } from '@/hooks/useLogs';
import { formatTime } from '@/lib/utils';
import type { LogLevel } from '@/types';

interface LogsProps {
  isOpen: boolean;
  onClose: () => void;
}

const LOG_LEVELS: { value: LogLevel | 'ALL'; label: string }[] = [
  { value: 'ALL', label: 'Все' },
  { value: 'DEBUG', label: 'Debug' },
  { value: 'INFO', label: 'Info' },
  { value: 'WARNING', label: 'Warning' },
  { value: 'ERROR', label: 'Error' },
];

const LOG_LEVEL_CLASSES: Record<LogLevel, string> = {
  DEBUG: 'log-debug',
  INFO: 'log-info',
  WARNING: 'log-warning',
  ERROR: 'log-error',
};

export function Logs({ isOpen, onClose }: LogsProps) {
  const {
    logs,
    filter,
    setFilter,
    autoScroll,
    setAutoScroll,
    logsEndRef,
    clearLogs,
    exportToFile,
    copyToClipboard,
  } = useLogs();

  if (!isOpen) return null;

  const handleCopy = async () => {
    try {
      await copyToClipboard();
    } catch (error) {
      console.error('Failed to copy logs:', error);
    }
  };

  const handleExport = async () => {
    try {
      const filePath = await exportToFile();
      console.log('Logs exported to:', filePath);
    } catch (error) {
      console.error('Failed to export logs:', error);
    }
  };

  const handleClear = async () => {
    if (confirm('Вы уверены, что хотите очистить все логи?')) {
      await clearLogs();
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-card rounded-lg shadow-lg w-[800px] h-[600px] max-w-[95vw] max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-border">
          <h2 className="text-lg font-semibold">Логи приложения</h2>
          <div className="flex items-center gap-2">
            <button
              onClick={handleCopy}
              className="p-2 rounded hover:bg-muted transition-colors"
              title="Копировать"
            >
              <Copy className="w-4 h-4" />
            </button>
            <button
              onClick={onClose}
              className="p-1 rounded hover:bg-muted transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Toolbar */}
        <div className="flex items-center gap-4 p-4 border-b border-border">
          <div className="flex items-center gap-2">
            <label className="text-sm text-muted-foreground">Уровень:</label>
            <select
              value={filter}
              onChange={(e) => setFilter(e.target.value as LogLevel | 'ALL')}
              className="px-2 py-1 rounded border border-border bg-background text-sm focus:outline-none focus:ring-1 focus:ring-ring"
            >
              {LOG_LEVELS.map((level) => (
                <option key={level.value} value={level.value}>
                  {level.label}
                </option>
              ))}
            </select>
          </div>

          <button
            onClick={handleClear}
            className="flex items-center gap-1 px-3 py-1.5 text-sm rounded hover:bg-muted transition-colors"
          >
            <Trash2 className="w-4 h-4" />
            <span>Очистить</span>
          </button>

          <button
            onClick={handleExport}
            className="flex items-center gap-1 px-3 py-1.5 text-sm rounded hover:bg-muted transition-colors"
          >
            <Download className="w-4 h-4" />
            <span>Экспорт в файл</span>
          </button>
        </div>

        {/* Logs */}
        <div className="flex-1 overflow-y-auto p-4 font-mono text-sm scrollbar-thin bg-muted/30">
          {logs.length === 0 ? (
            <div className="text-center text-muted-foreground py-8">
              Нет логов для отображения
            </div>
          ) : (
            <div className="space-y-1">
              {logs.map((log) => (
                <LogLine key={log.id} log={log} />
              ))}
              <div ref={logsEndRef} />
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-3 border-t border-border">
          <span className="text-sm text-muted-foreground">
            {logs.length} записей
          </span>
          <label className="flex items-center gap-2 text-sm cursor-pointer">
            <input
              type="checkbox"
              checked={autoScroll}
              onChange={(e) => setAutoScroll(e.target.checked)}
              className="w-4 h-4 rounded"
            />
            <ArrowDown className="w-4 h-4" />
            <span>Автопрокрутка вниз</span>
          </label>
        </div>
      </div>
    </div>
  );
}

interface LogLineProps {
  log: {
    id: number;
    level: LogLevel;
    message: string;
    context: string | null;
    created_at: string;
  };
}

function LogLine({ log }: LogLineProps) {
  const levelClass = LOG_LEVEL_CLASSES[log.level];

  return (
    <div className="flex gap-2 py-0.5 hover:bg-muted/50 rounded px-1">
      <span className="text-muted-foreground whitespace-nowrap">
        [{formatTime(log.created_at)}]
      </span>
      <span className={`${levelClass} font-medium w-16`}>
        {log.level}
      </span>
      <span className="flex-1 break-all">{log.message}</span>
    </div>
  );
}

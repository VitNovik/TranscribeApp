import { useState, useEffect, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { LogEntry, LogLevel, LogEntryEvent } from '@/types';
import * as api from '@/services/api';

const MAX_LOGS = 1000;

export function useLogs() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<LogLevel | 'ALL'>('ALL');
  const [autoScroll, setAutoScroll] = useState(true);
  const logsEndRef = useRef<HTMLDivElement>(null);

  const fetchLogs = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const levelFilter = filter === 'ALL' ? undefined : filter;
      const data = await api.getLogs(levelFilter, MAX_LOGS);
      setLogs(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch logs');
    } finally {
      setLoading(false);
    }
  }, [filter]);

  useEffect(() => {
    fetchLogs();
  }, [fetchLogs]);

  useEffect(() => {
    const unlisten = listen<LogEntryEvent>('log:entry', (event) => {
      const newLog: LogEntry = {
        id: Date.now(),
        level: event.payload.level,
        message: event.payload.message,
        context: event.payload.context ? JSON.stringify(event.payload.context) : null,
        created_at: event.payload.timestamp,
      };

      setLogs(prev => {
        const updated = [...prev, newLog];
        if (updated.length > MAX_LOGS) {
          return updated.slice(-MAX_LOGS);
        }
        return updated;
      });
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  useEffect(() => {
    if (autoScroll && logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);

  const clearLogs = useCallback(async () => {
    try {
      await api.clearLogs();
      setLogs([]);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to clear logs');
    }
  }, []);

  const exportToFile = useCallback(async () => {
    try {
      const filePath = await api.exportLogsToFile();
      return filePath;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to export logs');
      throw err;
    }
  }, []);

  const copyToClipboard = useCallback(async () => {
    try {
      await api.copyLogsToClipboard();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to copy logs');
      throw err;
    }
  }, []);

  const filteredLogs = filter === 'ALL'
    ? logs
    : logs.filter(log => log.level === filter);

  return {
    logs: filteredLogs,
    allLogs: logs,
    loading,
    error,
    filter,
    setFilter,
    autoScroll,
    setAutoScroll,
    logsEndRef,
    clearLogs,
    exportToFile,
    copyToClipboard,
    refetch: fetchLogs,
  };
}

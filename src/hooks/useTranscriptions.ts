import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import type {
  TranscriptionSummary,
  Transcription,
  ProcessingState,
  ProgressEvent,
  TranscriptionCompleteEvent,
  TranscriptionErrorEvent,
} from '@/types';
import * as api from '@/services/api';

export function useTranscriptions() {
  const [transcriptions, setTranscriptions] = useState<TranscriptionSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchTranscriptions = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.getAllTranscriptions();
      setTranscriptions(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch transcriptions');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchTranscriptions();
  }, [fetchTranscriptions]);

  const deleteTranscription = useCallback(async (id: number) => {
    try {
      await api.deleteTranscription(id);
      setTranscriptions(prev => prev.filter(t => t.id !== id));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete transcription');
      throw err;
    }
  }, []);

  const searchTranscriptions = useCallback(async (query: string) => {
    if (!query.trim()) {
      return fetchTranscriptions();
    }
    try {
      setLoading(true);
      const data = await api.searchTranscriptions(query);
      setTranscriptions(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to search transcriptions');
    } finally {
      setLoading(false);
    }
  }, [fetchTranscriptions]);

  return {
    transcriptions,
    loading,
    error,
    refetch: fetchTranscriptions,
    deleteTranscription,
    searchTranscriptions,
  };
}

export function useTranscription(id: number | null) {
  const [transcription, setTranscription] = useState<Transcription | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (id === null) {
      setTranscription(null);
      return;
    }

    const fetchTranscription = async () => {
      try {
        setLoading(true);
        setError(null);
        const data = await api.getTranscription(id);
        setTranscription(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch transcription');
      } finally {
        setLoading(false);
      }
    };

    fetchTranscription();
  }, [id]);

  return { transcription, loading, error };
}

export function useProcessing() {
  const [state, setState] = useState<ProcessingState>({
    isProcessing: false,
    currentJobId: null,
    stage: null,
    progress: 0,
    eta: 0,
  });
  const [error, setError] = useState<string | null>(null);
  const [completedId, setCompletedId] = useState<number | null>(null);

  useEffect(() => {
    const unlistenProgress = listen<ProgressEvent>('transcription:progress', (event) => {
      setState({
        isProcessing: true,
        currentJobId: event.payload.job_id,
        stage: event.payload.stage,
        progress: event.payload.progress,
        eta: event.payload.eta_seconds,
      });
    });

    const unlistenComplete = listen<TranscriptionCompleteEvent>('transcription:complete', (event) => {
      setState({
        isProcessing: false,
        currentJobId: null,
        stage: null,
        progress: 100,
        eta: 0,
      });
      setCompletedId(event.payload.transcription_id);
    });

    const unlistenError = listen<TranscriptionErrorEvent>('transcription:error', (event) => {
      setState({
        isProcessing: false,
        currentJobId: null,
        stage: null,
        progress: 0,
        eta: 0,
      });
      setError(event.payload.error);
    });

    return () => {
      unlistenProgress.then(fn => fn());
      unlistenComplete.then(fn => fn());
      unlistenError.then(fn => fn());
    };
  }, []);

  const startProcessing = useCallback(async (filePath: string) => {
    try {
      setError(null);
      setCompletedId(null);
      const job = await api.uploadFile(filePath);
      setState({
        isProcessing: true,
        currentJobId: job.job_id,
        stage: 'extracting',
        progress: 0,
        eta: 0,
      });
      return job;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start processing');
      throw err;
    }
  }, []);

  const cancelProcessing = useCallback(async () => {
    if (state.currentJobId) {
      try {
        await api.cancelProcessing(state.currentJobId);
        setState({
          isProcessing: false,
          currentJobId: null,
          stage: null,
          progress: 0,
          eta: 0,
        });
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to cancel processing');
      }
    }
  }, [state.currentJobId]);

  const clearCompleted = useCallback(() => {
    setCompletedId(null);
    setState({
      isProcessing: false,
      currentJobId: null,
      stage: null,
      progress: 0,
      eta: 0,
    });
  }, []);

  return {
    ...state,
    error,
    completedId,
    startProcessing,
    cancelProcessing,
    clearCompleted,
  };
}

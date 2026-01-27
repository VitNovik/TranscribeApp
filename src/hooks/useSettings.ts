import { useState, useEffect, useCallback } from 'react';
import type { Settings } from '@/types';
import * as api from '@/services/api';

const DEFAULT_SETTINGS: Settings = {
  whisper_model: 'medium',
  default_language: 'ru',
  auto_diarization: true,
  manual_speaker_count: null,
};

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    const fetchSettings = async () => {
      try {
        setLoading(true);
        setError(null);
        const data = await api.getSettings();
        setSettings(data);
      } catch (err) {
        // Use defaults if settings not found
        console.warn('Failed to fetch settings, using defaults:', err);
      } finally {
        setLoading(false);
      }
    };

    fetchSettings();
  }, []);

  const updateSettings = useCallback(async (newSettings: Partial<Settings>) => {
    const updated = { ...settings, ...newSettings };
    try {
      setSaving(true);
      setError(null);
      await api.updateSettings(updated);
      setSettings(updated);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save settings');
      throw err;
    } finally {
      setSaving(false);
    }
  }, [settings]);

  return {
    settings,
    loading,
    saving,
    error,
    updateSettings,
  };
}

export function useAvailableModels() {
  const [models, setModels] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [downloading, setDownloading] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchModels = async () => {
      try {
        setLoading(true);
        const data = await api.getAvailableModels();
        setModels(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch models');
      } finally {
        setLoading(false);
      }
    };

    fetchModels();
  }, []);

  const downloadModel = useCallback(async (modelName: string) => {
    try {
      setDownloading(modelName);
      setError(null);
      await api.downloadModel(modelName);
      setModels(prev => [...prev, modelName]);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to download model');
      throw err;
    } finally {
      setDownloading(null);
    }
  }, []);

  return {
    models,
    loading,
    downloading,
    error,
    downloadModel,
  };
}

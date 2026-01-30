import { useState, useEffect } from 'react';
import { X, Download, Loader2, Check, HardDrive } from 'lucide-react';
import { useSettings, useAvailableModels } from '@/hooks/useSettings';
import type { WhisperModel, Settings as SettingsType } from '@/types';
import { SUPPORTED_LANGUAGES } from '@/types';

interface SettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

const WHISPER_MODELS: { value: WhisperModel; label: string; description: string }[] = [
  { value: 'tiny', label: 'Tiny', description: 'Быстро, низкое качество' },
  { value: 'base', label: 'Base', description: 'Быстро' },
  { value: 'small', label: 'Small', description: 'Баланс скорости и качества' },
  { value: 'medium', label: 'Medium', description: 'Качественно (рекомендуется)' },
  { value: 'large-v3', label: 'Large-v3', description: 'Максимальное качество' },
];

export function Settings({ isOpen, onClose }: SettingsProps) {
  const { settings, loading, saving, updateSettings } = useSettings();
  const { models: availableModels, downloading, downloadModel } = useAvailableModels();
  const [localSettings, setLocalSettings] = useState<SettingsType>(settings);

  // Sync localSettings when settings are loaded from backend
  useEffect(() => {
    if (!loading) {
      setLocalSettings(settings);
    }
  }, [settings, loading]);

  if (!isOpen) return null;

  const handleChange = <K extends keyof SettingsType>(key: K, value: SettingsType[K]) => {
    setLocalSettings(prev => ({ ...prev, [key]: value }));
  };

  const handleSave = async () => {
    await updateSettings(localSettings);
    onClose();
  };

  const handleDownloadModel = async (model: string) => {
    try {
      await downloadModel(model);
    } catch (error) {
      console.error('Failed to download model:', error);
    }
  };

  const isModelAvailable = (model: string) => availableModels.includes(model);

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-card rounded-lg shadow-lg w-[500px] max-w-[90vw] max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between p-4 border-b border-border sticky top-0 bg-card">
          <h2 className="text-lg font-semibold">Настройки</h2>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-muted transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {loading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <div className="p-6 space-y-6">
            {/* Whisper Model */}
            <div className="space-y-3">
              <h3 className="font-medium">Модель Whisper</h3>
              <div className="space-y-2">
                {WHISPER_MODELS.map((model) => (
                  <label
                    key={model.value}
                    className="flex items-center gap-3 p-3 rounded-lg border border-border hover:bg-muted/50 cursor-pointer"
                  >
                    <input
                      type="radio"
                      name="whisper_model"
                      value={model.value}
                      checked={localSettings.whisper_model === model.value}
                      onChange={() => handleChange('whisper_model', model.value)}
                      className="w-4 h-4"
                    />
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">{model.label}</span>
                        {!isModelAvailable(model.value) && (
                          <button
                            onClick={(e) => {
                              e.preventDefault();
                              handleDownloadModel(model.value);
                            }}
                            disabled={downloading === model.value}
                            className="text-xs px-2 py-0.5 rounded bg-primary text-primary-foreground hover:opacity-90 disabled:opacity-50"
                          >
                            {downloading === model.value ? (
                              <Loader2 className="w-3 h-3 animate-spin" />
                            ) : (
                              <Download className="w-3 h-3" />
                            )}
                          </button>
                        )}
                        {isModelAvailable(model.value) && (
                          <Check className="w-4 h-4 text-green-500" />
                        )}
                      </div>
                      <p className="text-sm text-muted-foreground">{model.description}</p>
                    </div>
                  </label>
                ))}
              </div>
            </div>

            {/* Default Language */}
            <div className="space-y-3">
              <h3 className="font-medium">Язык по умолчанию</h3>
              <select
                value={localSettings.default_language}
                onChange={(e) => handleChange('default_language', e.target.value)}
                className="w-full px-3 py-2 rounded-lg border border-border bg-background focus:outline-none focus:ring-2 focus:ring-ring"
              >
                {SUPPORTED_LANGUAGES.map((lang) => (
                  <option key={lang.code} value={lang.code}>
                    {lang.name}
                  </option>
                ))}
              </select>
            </div>

            {/* Diarization */}
            <div className="space-y-3">
              <h3 className="font-medium">Диаризация</h3>
              <label className="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={localSettings.auto_diarization}
                  onChange={(e) => handleChange('auto_diarization', e.target.checked)}
                  className="w-4 h-4 rounded"
                />
                <span>Автоматически определять спикеров</span>
              </label>
              <label className="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={localSettings.manual_speaker_count !== null}
                  onChange={(e) =>
                    handleChange('manual_speaker_count', e.target.checked ? 2 : null)
                  }
                  className="w-4 h-4 rounded"
                />
                <span>Указывать количество спикеров вручную:</span>
                <input
                  type="number"
                  min="1"
                  max="20"
                  value={localSettings.manual_speaker_count ?? 2}
                  onChange={(e) =>
                    handleChange('manual_speaker_count', parseInt(e.target.value, 10) || null)
                  }
                  disabled={localSettings.manual_speaker_count === null}
                  className="w-16 px-2 py-1 rounded border border-border bg-background focus:outline-none focus:ring-1 focus:ring-ring disabled:opacity-50"
                />
              </label>
            </div>

            {/* Storage Info */}
            <div className="space-y-3">
              <h3 className="font-medium flex items-center gap-2">
                <HardDrive className="w-4 h-4" />
                Хранение
              </h3>
              <div className="text-sm text-muted-foreground space-y-1">
                <p>База данных: ~/.transcribe/data.db</p>
                <p>Модели: ~/.transcribe/models/</p>
              </div>
              <div className="flex gap-2">
                <button className="px-3 py-1.5 text-sm rounded-lg border border-border hover:bg-muted transition-colors">
                  Очистить кэш
                </button>
                <button className="px-3 py-1.5 text-sm rounded-lg border border-border hover:bg-muted transition-colors">
                  Экспорт настроек
                </button>
              </div>
            </div>
          </div>
        )}

        <div className="flex justify-end gap-2 p-4 border-t border-border sticky bottom-0 bg-card">
          <button
            onClick={onClose}
            disabled={saving}
            className="px-4 py-2 rounded-lg hover:bg-muted transition-colors"
          >
            Отмена
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:opacity-90 disabled:opacity-50"
          >
            {saving ? 'Сохранение...' : 'Сохранить'}
          </button>
        </div>
      </div>
    </div>
  );
}

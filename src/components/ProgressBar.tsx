import { X, Check, Loader2, AlertCircle } from 'lucide-react';
import type { ProcessingStage } from '@/types';

interface ProgressBarProps {
  stage: ProcessingStage | null;
  progress: number;
  eta: number;
  fileName?: string;
  onCancel: () => void;
  error?: string | null;
  completed?: boolean;
  onComplete?: () => void;
}

const STAGE_LABELS: Record<ProcessingStage, string> = {
  extracting: 'Извлечение аудио',
  transcribing: 'Транскрибация',
  diarizing: 'Диаризация',
};

export function ProgressBar({
  stage,
  progress,
  eta,
  fileName,
  onCancel,
  error,
  completed,
  onComplete,
}: ProgressBarProps) {
  const formatEta = (seconds: number): string => {
    if (seconds < 60) {
      return `${Math.ceil(seconds)} сек`;
    }
    const minutes = Math.floor(seconds / 60);
    const secs = Math.ceil(seconds % 60);
    return `${minutes} мин ${secs} сек`;
  };

  if (error) {
    return (
      <div className="fixed bottom-4 right-4 w-96 bg-card rounded-lg shadow-lg border border-destructive p-4">
        <div className="flex items-start gap-3">
          <AlertCircle className="w-5 h-5 text-destructive flex-shrink-0 mt-0.5" />
          <div className="flex-1 min-w-0">
            <p className="font-medium text-destructive">Ошибка обработки</p>
            <p className="text-sm text-muted-foreground mt-1 line-clamp-2">{error}</p>
          </div>
          <button
            onClick={onCancel}
            className="p-1 rounded hover:bg-muted transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>
    );
  }

  if (completed) {
    return (
      <div className="fixed bottom-4 right-4 w-96 bg-card rounded-lg shadow-lg border border-green-500 p-4">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-full bg-green-500 flex items-center justify-center">
            <Check className="w-5 h-5 text-white" />
          </div>
          <div className="flex-1 min-w-0">
            <p className="font-medium">Транскрибация завершена!</p>
            {fileName && (
              <p className="text-sm text-muted-foreground truncate">{fileName}</p>
            )}
          </div>
          <button
            onClick={onComplete}
            className="px-3 py-1.5 rounded-lg bg-primary text-primary-foreground text-sm hover:opacity-90"
          >
            Открыть
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed bottom-4 right-4 w-96 bg-card rounded-lg shadow-lg border border-border p-4">
      <div className="flex items-center justify-between mb-2">
        <span className="font-medium">Обработка файла...</span>
        <button
          onClick={onCancel}
          className="p-1 rounded hover:bg-muted transition-colors"
          title="Отменить"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      {fileName && (
        <p className="text-sm text-muted-foreground mb-3 truncate">{fileName}</p>
      )}

      <div className="space-y-2">
        <StageItem
          label="Извлечение аудио"
          isActive={stage === 'extracting'}
          isComplete={stage !== 'extracting' && (stage === 'transcribing' || stage === 'diarizing')}
          progress={stage === 'extracting' ? progress : undefined}
        />
        <StageItem
          label="Транскрибация"
          isActive={stage === 'transcribing'}
          isComplete={stage === 'diarizing'}
          progress={stage === 'transcribing' ? progress : undefined}
        />
        <StageItem
          label="Диаризация"
          isActive={stage === 'diarizing'}
          isComplete={false}
          progress={stage === 'diarizing' ? progress : undefined}
        />
      </div>

      {stage && (
        <div className="mt-3">
          <div className="progress-bar">
            <div
              className="progress-bar-fill"
              style={{ width: `${progress}%` }}
            />
          </div>
          <div className="flex justify-between text-xs text-muted-foreground mt-1">
            <span>{STAGE_LABELS[stage]}... {Math.round(progress)}%</span>
            {eta > 0 && <span>~{formatEta(eta)}</span>}
          </div>
        </div>
      )}
    </div>
  );
}

interface StageItemProps {
  label: string;
  isActive: boolean;
  isComplete: boolean;
  progress?: number;
}

function StageItem({ label, isActive, isComplete, progress }: StageItemProps) {
  return (
    <div className="flex items-center gap-2">
      <div className="w-5 h-5 flex items-center justify-center">
        {isComplete ? (
          <Check className="w-4 h-4 text-green-500" />
        ) : isActive ? (
          <Loader2 className="w-4 h-4 animate-spin text-primary" />
        ) : (
          <div className="w-2 h-2 rounded-full bg-muted-foreground/30" />
        )}
      </div>
      <span className={`text-sm ${isActive ? 'text-foreground' : isComplete ? 'text-muted-foreground' : 'text-muted-foreground/50'}`}>
        {label}
      </span>
      {isActive && progress !== undefined && (
        <span className="text-xs text-muted-foreground ml-auto">{Math.round(progress)}%</span>
      )}
    </div>
  );
}

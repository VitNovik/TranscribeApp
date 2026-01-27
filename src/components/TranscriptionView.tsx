import { ArrowLeft, Download, Trash2, Loader2, Clock, Calendar, FileAudio, Globe, Cpu } from 'lucide-react';
import { useTranscription } from '@/hooks/useTranscriptions';
import { useSegmentsWithSpeakers } from '@/hooks/useSpeakers';
import { SpeakersPanel } from './SpeakersPanel';
import { formatTimestamp, formatDuration, formatDate, formatTime, getFileName } from '@/lib/utils';
import type { SegmentWithSpeaker } from '@/types';

interface TranscriptionViewProps {
  transcriptionId: number;
  onBack: () => void;
  onDelete: () => void;
}

export function TranscriptionView({ transcriptionId, onBack, onDelete }: TranscriptionViewProps) {
  const { transcription, loading, error } = useTranscription(transcriptionId);
  const {
    segments,
    speakers,
    loading: segmentsLoading,
    updateSpeakerName,
  } = useSegmentsWithSpeakers(transcriptionId);

  const handleExport = async (format: string) => {
    // TODO: Implement export
    console.log('Export as:', format);
  };

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center">
        <Loader2 className="w-8 h-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (error || !transcription) {
    return (
      <div className="h-full flex flex-col items-center justify-center text-muted-foreground">
        <p>Не удалось загрузить транскрипцию</p>
        <button
          onClick={onBack}
          className="mt-4 px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:opacity-90"
        >
          Назад
        </button>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-border">
        <div className="flex items-center gap-4">
          <button
            onClick={onBack}
            className="p-2 rounded-lg hover:bg-muted transition-colors"
          >
            <ArrowLeft className="w-5 h-5" />
          </button>
          <h1 className="text-xl font-semibold">{transcription.title}</h1>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => handleExport('txt')}
            className="flex items-center gap-2 px-3 py-2 rounded-lg hover:bg-muted transition-colors"
          >
            <Download className="w-4 h-4" />
            <span>Экспорт</span>
          </button>
          <button
            onClick={onDelete}
            className="flex items-center gap-2 px-3 py-2 rounded-lg text-destructive hover:bg-destructive/10 transition-colors"
          >
            <Trash2 className="w-4 h-4" />
            <span>Удалить</span>
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Transcript */}
        <div className="flex-1 overflow-y-auto p-4 scrollbar-thin">
          {segmentsLoading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
            </div>
          ) : segments.length > 0 ? (
            <div className="space-y-4">
              {segments.map((segment) => (
                <SegmentItem key={segment.id} segment={segment} />
              ))}
            </div>
          ) : (
            <div className="bg-muted/50 rounded-lg p-6">
              <p className="whitespace-pre-wrap text-foreground">
                {transcription.full_text}
              </p>
            </div>
          )}
        </div>

        {/* Sidebar */}
        <div className="w-80 border-l border-border overflow-y-auto scrollbar-thin">
          <div className="p-4 space-y-6">
            {/* Info */}
            <div className="space-y-3">
              <h2 className="font-semibold flex items-center gap-2">
                <span>Информация</span>
              </h2>
              <div className="space-y-2 text-sm">
                <div className="flex items-center gap-2 text-muted-foreground">
                  <Calendar className="w-4 h-4" />
                  <span>{formatDate(transcription.created_at)}</span>
                </div>
                <div className="flex items-center gap-2 text-muted-foreground">
                  <Clock className="w-4 h-4" />
                  <span>{formatTime(transcription.created_at)}</span>
                </div>
                <div className="flex items-center gap-2 text-muted-foreground">
                  <Clock className="w-4 h-4" />
                  <span>{formatDuration(transcription.duration)}</span>
                </div>
              </div>
            </div>

            {/* Speakers */}
            <SpeakersPanel
              speakers={speakers}
              onUpdateName={updateSpeakerName}
            />

            {/* Metadata */}
            <div className="space-y-3">
              <h2 className="font-semibold">Метаданные</h2>
              <div className="space-y-2 text-sm">
                <div className="flex items-center gap-2 text-muted-foreground">
                  <FileAudio className="w-4 h-4" />
                  <span className="truncate" title={transcription.file_path}>
                    {getFileName(transcription.file_path)}
                  </span>
                </div>
                <div className="flex items-center gap-2 text-muted-foreground">
                  <Globe className="w-4 h-4" />
                  <span>Язык: {transcription.language}</span>
                </div>
                <div className="flex items-center gap-2 text-muted-foreground">
                  <Cpu className="w-4 h-4" />
                  <span>Модель: Whisper {transcription.model}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

interface SegmentItemProps {
  segment: SegmentWithSpeaker;
}

function SegmentItem({ segment }: SegmentItemProps) {
  return (
    <div className="segment-item">
      <div className="flex items-start gap-3">
        <span className="text-xs font-mono text-muted-foreground whitespace-nowrap">
          [{formatTimestamp(segment.start_time)}]
        </span>
        {segment.speaker && (
          <span className="flex items-center gap-1 text-sm font-medium whitespace-nowrap">
            <span
              className="speaker-dot"
              style={{ backgroundColor: segment.speaker.color }}
            />
            {segment.speaker.display_name}
          </span>
        )}
      </div>
      <p className="mt-1 text-foreground pl-20">{segment.text}</p>
    </div>
  );
}

import { useState, useCallback, useEffect } from 'react';
import { ScrollText } from 'lucide-react';
import {
  TranscriptionList,
  TranscriptionView,
  FileUpload,
  ProgressBar,
  Settings,
  Logs,
} from '@/components';
import { useTranscriptions, useProcessing } from '@/hooks/useTranscriptions';
import { getFileName } from '@/lib/utils';

type View = 'list' | 'detail';

export default function App() {
  const [view, setView] = useState<View>('list');
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [showUpload, setShowUpload] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showLogs, setShowLogs] = useState(false);
  const [currentFileName, setCurrentFileName] = useState<string>('');

  const { refetch, deleteTranscription } = useTranscriptions();
  const {
    isProcessing,
    stage,
    progress,
    eta,
    error,
    completedId,
    startProcessing,
    cancelProcessing,
    clearCompleted,
  } = useProcessing();

  const handleSelectTranscription = useCallback((id: number) => {
    setSelectedId(id);
    setView('detail');
  }, []);

  const handleBack = useCallback(() => {
    setView('list');
    setSelectedId(null);
  }, []);

  const handleDeleteTranscription = useCallback(async () => {
    if (selectedId !== null) {
      await deleteTranscription(selectedId);
      handleBack();
    }
  }, [selectedId, deleteTranscription, handleBack]);

  const handleFileSelect = useCallback(async (filePath: string) => {
    setShowUpload(false);
    setCurrentFileName(getFileName(filePath));
    try {
      await startProcessing(filePath);
    } catch (error) {
      console.error('Failed to start processing:', error);
    }
  }, [startProcessing]);

  const handleComplete = useCallback(() => {
    if (completedId !== null) {
      handleSelectTranscription(completedId);
      clearCompleted();
      refetch();
    }
  }, [completedId, handleSelectTranscription, clearCompleted, refetch]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd/Ctrl + N - New transcription
      if ((e.metaKey || e.ctrlKey) && e.key === 'n') {
        e.preventDefault();
        setShowUpload(true);
      }
      // Cmd/Ctrl + , - Settings
      if ((e.metaKey || e.ctrlKey) && e.key === ',') {
        e.preventDefault();
        setShowSettings(true);
      }
      // Cmd/Ctrl + L - Logs
      if ((e.metaKey || e.ctrlKey) && e.key === 'l') {
        e.preventDefault();
        setShowLogs(true);
      }
      // Escape - Close modals or go back
      if (e.key === 'Escape') {
        if (showUpload) setShowUpload(false);
        else if (showSettings) setShowSettings(false);
        else if (showLogs) setShowLogs(false);
        else if (view === 'detail') handleBack();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [showUpload, showSettings, showLogs, view, handleBack]);

  return (
    <div className="h-screen flex flex-col">
      {/* App content */}
      <div className="flex-1 flex overflow-hidden">
        {view === 'list' ? (
          <div className="flex-1">
            <TranscriptionList
              onSelect={handleSelectTranscription}
              onNewTranscription={() => setShowUpload(true)}
              onOpenSettings={() => setShowSettings(true)}
              selectedId={selectedId}
            />
          </div>
        ) : selectedId !== null ? (
          <TranscriptionView
            transcriptionId={selectedId}
            onBack={handleBack}
            onDelete={handleDeleteTranscription}
          />
        ) : null}
      </div>

      {/* Footer with logs button */}
      <div className="h-8 border-t border-border flex items-center justify-between px-4 text-xs text-muted-foreground bg-muted/30">
        <span>TranscribeApp v1.0.0</span>
        <button
          onClick={() => setShowLogs(true)}
          className="flex items-center gap-1 hover:text-foreground transition-colors"
        >
          <ScrollText className="w-3 h-3" />
          <span>Логи</span>
        </button>
      </div>

      {/* Modals */}
      {showUpload && (
        <FileUpload
          onFileSelect={handleFileSelect}
          onClose={() => setShowUpload(false)}
        />
      )}

      <Settings isOpen={showSettings} onClose={() => setShowSettings(false)} />
      <Logs isOpen={showLogs} onClose={() => setShowLogs(false)} />

      {/* Progress indicator */}
      {(isProcessing || error || completedId !== null) && (
        <ProgressBar
          stage={stage}
          progress={progress}
          eta={eta}
          fileName={currentFileName}
          onCancel={() => {
            cancelProcessing();
            clearCompleted();
          }}
          error={error}
          completed={completedId !== null}
          onComplete={handleComplete}
        />
      )}
    </div>
  );
}

import { useState, useCallback } from 'react';
import { Upload, FileAudio, FileVideo, X } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { AUDIO_FORMATS, VIDEO_FORMATS, SUPPORTED_FORMATS } from '@/types';
import { getFileName, getFileExtension } from '@/lib/utils';

interface FileUploadProps {
  onFileSelect: (path: string) => void;
  onClose: () => void;
}

export function FileUpload({ onFileSelect, onClose }: FileUploadProps) {
  const [isDragging, setIsDragging] = useState(false);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);

  const handleSelectFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'Media Files',
            extensions: SUPPORTED_FORMATS.map(f => f.slice(1)),
          },
          {
            name: 'Audio',
            extensions: AUDIO_FORMATS.map(f => f.slice(1)),
          },
          {
            name: 'Video',
            extensions: VIDEO_FORMATS.map(f => f.slice(1)),
          },
        ],
      });

      if (selected && typeof selected === 'string') {
        setSelectedFile(selected);
      }
    } catch (error) {
      console.error('Failed to select file:', error);
    }
  };

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) {
      const file = files[0];
      const ext = getFileExtension(file.name);
      if (SUPPORTED_FORMATS.includes(ext as typeof SUPPORTED_FORMATS[number])) {
        // Note: In Tauri, we need the actual file path, not the File object
        // This is a limitation of drag-and-drop in Tauri
        console.log('File dropped:', file.name);
      }
    }
  }, []);

  const handleStart = () => {
    if (selectedFile) {
      onFileSelect(selectedFile);
    }
  };

  const isAudio = selectedFile && AUDIO_FORMATS.some(f => selectedFile.toLowerCase().endsWith(f));
  const FileIcon = isAudio ? FileAudio : FileVideo;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-card rounded-lg shadow-lg p-6 w-[480px] max-w-[90vw]">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-semibold">Добавить файл для транскрибации</h2>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-muted transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {selectedFile ? (
          <div className="space-y-4">
            <div className="flex items-center gap-3 p-4 rounded-lg bg-muted/50">
              <FileIcon className="w-8 h-8 text-primary" />
              <div className="flex-1 min-w-0">
                <p className="font-medium truncate">{getFileName(selectedFile)}</p>
                <p className="text-sm text-muted-foreground truncate">{selectedFile}</p>
              </div>
              <button
                onClick={() => setSelectedFile(null)}
                className="p-1 rounded hover:bg-muted transition-colors"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <div className="flex justify-end gap-2">
              <button
                onClick={() => setSelectedFile(null)}
                className="px-4 py-2 rounded-lg hover:bg-muted transition-colors"
              >
                Выбрать другой
              </button>
              <button
                onClick={handleStart}
                className="px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:opacity-90"
              >
                Начать транскрибацию
              </button>
            </div>
          </div>
        ) : (
          <div
            onClick={handleSelectFile}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
            className={`
              border-2 border-dashed rounded-lg p-12 text-center cursor-pointer transition-colors
              ${isDragging ? 'border-primary bg-primary/5' : 'border-border hover:border-muted-foreground'}
            `}
          >
            <Upload className="w-12 h-12 mx-auto mb-4 text-muted-foreground" />
            <p className="text-lg mb-2">Перетащите файл сюда</p>
            <p className="text-muted-foreground mb-4">или нажмите для выбора</p>
            <div className="text-sm text-muted-foreground space-y-1">
              <p>Поддерживаемые форматы:</p>
              <p>Аудио: {AUDIO_FORMATS.map(f => f.slice(1)).join(', ')}</p>
              <p>Видео: {VIDEO_FORMATS.map(f => f.slice(1)).join(', ')}</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

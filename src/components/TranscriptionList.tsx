import { useState } from 'react';
import { Search, FileText, Plus, Settings, Trash2, Loader2 } from 'lucide-react';
import { useTranscriptions } from '@/hooks/useTranscriptions';
import { formatDuration, formatDate, speakerLabel, debounce } from '@/lib/utils';
import type { TranscriptionSummary } from '@/types';

interface TranscriptionListProps {
  onSelect: (id: number) => void;
  onNewTranscription: () => void;
  onOpenSettings: () => void;
  selectedId: number | null;
}

export function TranscriptionList({
  onSelect,
  onNewTranscription,
  onOpenSettings,
  selectedId,
}: TranscriptionListProps) {
  const { transcriptions, loading, error, deleteTranscription, searchTranscriptions } = useTranscriptions();
  const [searchQuery, setSearchQuery] = useState('');
  const [deleteConfirm, setDeleteConfirm] = useState<number | null>(null);

  const handleSearch = debounce((query: string) => {
    searchTranscriptions(query);
  }, 300);

  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const query = e.target.value;
    setSearchQuery(query);
    handleSearch(query);
  };

  const handleDelete = async (id: number, e: React.MouseEvent) => {
    e.stopPropagation();
    if (deleteConfirm === id) {
      await deleteTranscription(id);
      setDeleteConfirm(null);
    } else {
      setDeleteConfirm(id);
      setTimeout(() => setDeleteConfirm(null), 3000);
    }
  };

  return (
    <div className="h-full flex flex-col bg-background">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-border">
        <h1 className="text-xl font-semibold">TranscribeApp</h1>
        <div className="flex gap-2">
          <button
            onClick={onNewTranscription}
            className="p-2 rounded-lg hover:bg-muted transition-colors"
            title="Новая транскрипция"
          >
            <Plus className="w-5 h-5" />
          </button>
          <button
            onClick={onOpenSettings}
            className="p-2 rounded-lg hover:bg-muted transition-colors"
            title="Настройки"
          >
            <Settings className="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* Search */}
      <div className="p-4">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            type="text"
            placeholder="Поиск..."
            value={searchQuery}
            onChange={handleSearchChange}
            className="w-full pl-10 pr-4 py-2 rounded-lg border border-border bg-background focus:outline-none focus:ring-2 focus:ring-ring"
          />
        </div>
      </div>

      {/* List */}
      <div className="flex-1 overflow-y-auto scrollbar-thin">
        {loading && (
          <div className="flex items-center justify-center py-8">
            <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
          </div>
        )}

        {error && (
          <div className="p-4 text-center text-destructive">
            {error}
          </div>
        )}

        {!loading && !error && transcriptions.length === 0 && (
          <div className="p-8 text-center text-muted-foreground">
            <FileText className="w-12 h-12 mx-auto mb-4 opacity-50" />
            <p>Нет транскрипций</p>
            <p className="text-sm mt-2">
              Нажмите + чтобы добавить новый файл
            </p>
          </div>
        )}

        {!loading && !error && transcriptions.map((item) => (
          <TranscriptionItem
            key={item.id}
            item={item}
            isSelected={selectedId === item.id}
            isDeleteConfirm={deleteConfirm === item.id}
            onSelect={() => onSelect(item.id)}
            onDelete={(e) => handleDelete(item.id, e)}
          />
        ))}
      </div>
    </div>
  );
}

interface TranscriptionItemProps {
  item: TranscriptionSummary;
  isSelected: boolean;
  isDeleteConfirm: boolean;
  onSelect: () => void;
  onDelete: (e: React.MouseEvent) => void;
}

function TranscriptionItem({
  item,
  isSelected,
  isDeleteConfirm,
  onSelect,
  onDelete,
}: TranscriptionItemProps) {
  return (
    <div
      onClick={onSelect}
      className={`p-4 border-b border-border cursor-pointer transition-colors group ${
        isSelected ? 'bg-muted' : 'hover:bg-muted/50'
      }`}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <FileText className="w-4 h-4 text-muted-foreground flex-shrink-0" />
            <h3 className="font-medium truncate">{item.title}</h3>
          </div>
          <div className="mt-1 text-sm text-muted-foreground">
            {speakerLabel(item.speaker_count)} • {item.language}
          </div>
        </div>
        <div className="flex items-center gap-2 ml-2">
          <div className="text-right">
            <div className="text-sm font-mono">{formatDuration(item.duration)}</div>
            <div className="text-xs text-muted-foreground">{formatDate(item.created_at)}</div>
          </div>
          <button
            onClick={onDelete}
            className={`p-1.5 rounded opacity-0 group-hover:opacity-100 transition-opacity ${
              isDeleteConfirm
                ? 'bg-destructive text-destructive-foreground opacity-100'
                : 'hover:bg-muted'
            }`}
            title={isDeleteConfirm ? 'Нажмите ещё раз для удаления' : 'Удалить'}
          >
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
}

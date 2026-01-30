import { useState } from 'react';
import { Users, Edit2, Check, X } from 'lucide-react';
import type { Speaker } from '@/types';

interface SpeakersPanelProps {
  speakers: Speaker[];
  onUpdateName: (speakerId: number, newName: string) => Promise<void>;
}

export function SpeakersPanel({ speakers, onUpdateName }: SpeakersPanelProps) {
  if (speakers.length === 0) {
    return null;
  }

  return (
    <div className="space-y-3">
      <h2 className="font-semibold flex items-center gap-2">
        <Users className="w-4 h-4" />
        <span>Спикеры</span>
      </h2>
      <div className="space-y-2">
        {speakers.map((speaker) => (
          <SpeakerItem
            key={speaker.id}
            speaker={speaker}
            onUpdateName={onUpdateName}
          />
        ))}
      </div>
    </div>
  );
}

interface SpeakerItemProps {
  speaker: Speaker;
  onUpdateName: (speakerId: number, newName: string) => Promise<void>;
}

function SpeakerItem({ speaker, onUpdateName }: SpeakerItemProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState(speaker.display_name);
  const [saving, setSaving] = useState(false);

  const handleEdit = () => {
    setEditValue(speaker.display_name);
    setIsEditing(true);
  };

  const handleCancel = () => {
    setEditValue(speaker.display_name);
    setIsEditing(false);
  };

  const handleSave = async () => {
    if (editValue.trim() === '' || editValue === speaker.display_name) {
      handleCancel();
      return;
    }

    try {
      setSaving(true);
      await onUpdateName(speaker.id, editValue.trim());
      setIsEditing(false);
    } catch (error) {
      console.error('Failed to update speaker name:', error);
    } finally {
      setSaving(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSave();
    } else if (e.key === 'Escape') {
      handleCancel();
    }
  };

  return (
    <div className="group flex items-center gap-2 p-2 rounded-lg bg-muted/50">
      <span
        className="speaker-dot flex-shrink-0"
        style={{ backgroundColor: speaker.color }}
      />

      {isEditing ? (
        <div className="flex-1 flex items-center gap-1">
          <input
            type="text"
            value={editValue}
            onChange={(e) => setEditValue(e.target.value)}
            onKeyDown={handleKeyDown}
            autoFocus
            disabled={saving}
            className="flex-1 px-2 py-1 text-sm rounded border border-border bg-background focus:outline-none focus:ring-1 focus:ring-ring"
          />
          <button
            onClick={handleSave}
            disabled={saving}
            className="p-1 rounded hover:bg-muted transition-colors text-primary"
          >
            <Check className="w-4 h-4" />
          </button>
          <button
            onClick={handleCancel}
            disabled={saving}
            className="p-1 rounded hover:bg-muted transition-colors text-muted-foreground"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      ) : (
        <>
          <span className="flex-1 text-sm">{speaker.display_name}</span>
          <button
            onClick={handleEdit}
            className="p-1 rounded hover:bg-muted transition-colors text-muted-foreground opacity-0 group-hover:opacity-100"
            title="Переименовать"
          >
            <Edit2 className="w-3.5 h-3.5" />
          </button>
        </>
      )}
    </div>
  );
}

interface RenameSpeakerDialogProps {
  speaker: Speaker;
  isOpen: boolean;
  onClose: () => void;
  onSave: (newName: string) => Promise<void>;
}

export function RenameSpeakerDialog({
  speaker,
  isOpen,
  onClose,
  onSave,
}: RenameSpeakerDialogProps) {
  const [name, setName] = useState(speaker.display_name);
  const [saving, setSaving] = useState(false);

  if (!isOpen) return null;

  const handleSave = async () => {
    if (name.trim() === '') return;

    try {
      setSaving(true);
      await onSave(name.trim());
      onClose();
    } catch (error) {
      console.error('Failed to save speaker name:', error);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-card rounded-lg shadow-lg p-6 w-80">
        <h2 className="text-lg font-semibold mb-4">Переименовать спикера</h2>

        <div className="space-y-4">
          <div>
            <label className="text-sm text-muted-foreground">
              Текущее имя: {speaker.display_name}
            </label>
          </div>

          <div>
            <label className="text-sm text-muted-foreground mb-1 block">
              Новое имя:
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSave()}
              autoFocus
              disabled={saving}
              className="w-full px-3 py-2 rounded-lg border border-border bg-background focus:outline-none focus:ring-2 focus:ring-ring"
            />
          </div>
        </div>

        <div className="flex justify-end gap-2 mt-6">
          <button
            onClick={onClose}
            disabled={saving}
            className="px-4 py-2 rounded-lg hover:bg-muted transition-colors"
          >
            Отмена
          </button>
          <button
            onClick={handleSave}
            disabled={saving || name.trim() === ''}
            className="px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:opacity-90 disabled:opacity-50"
          >
            {saving ? 'Сохранение...' : 'Сохранить'}
          </button>
        </div>
      </div>
    </div>
  );
}

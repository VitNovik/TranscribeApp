import { useState, useEffect, useCallback } from 'react';
import type { Speaker, Segment, SegmentWithSpeaker } from '@/types';
import * as api from '@/services/api';

export function useSpeakers(transcriptionId: number | null) {
  const [speakers, setSpeakers] = useState<Speaker[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (transcriptionId === null) {
      setSpeakers([]);
      return;
    }

    const fetchSpeakers = async () => {
      try {
        setLoading(true);
        setError(null);
        const data = await api.getSpeakers(transcriptionId);
        setSpeakers(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch speakers');
      } finally {
        setLoading(false);
      }
    };

    fetchSpeakers();
  }, [transcriptionId]);

  const updateSpeakerName = useCallback(async (speakerId: number, newName: string) => {
    try {
      await api.updateSpeakerName(speakerId, newName);
      setSpeakers(prev =>
        prev.map(s =>
          s.id === speakerId ? { ...s, display_name: newName } : s
        )
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update speaker name');
      throw err;
    }
  }, []);

  return {
    speakers,
    loading,
    error,
    updateSpeakerName,
  };
}

export function useSegments(transcriptionId: number | null) {
  const [segments, setSegments] = useState<Segment[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (transcriptionId === null) {
      setSegments([]);
      return;
    }

    const fetchSegments = async () => {
      try {
        setLoading(true);
        setError(null);
        const data = await api.getSegments(transcriptionId);
        setSegments(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to fetch segments');
      } finally {
        setLoading(false);
      }
    };

    fetchSegments();
  }, [transcriptionId]);

  return {
    segments,
    loading,
    error,
  };
}

export function useSegmentsWithSpeakers(transcriptionId: number | null) {
  const { segments, loading: segmentsLoading, error: segmentsError } = useSegments(transcriptionId);
  const { speakers, loading: speakersLoading, error: speakersError, updateSpeakerName } = useSpeakers(transcriptionId);

  const segmentsWithSpeakers: SegmentWithSpeaker[] = segments.map(segment => ({
    ...segment,
    speaker: speakers.find(s => s.id === segment.speaker_id),
  }));

  return {
    segments: segmentsWithSpeakers,
    speakers,
    loading: segmentsLoading || speakersLoading,
    error: segmentsError || speakersError,
    updateSpeakerName,
  };
}

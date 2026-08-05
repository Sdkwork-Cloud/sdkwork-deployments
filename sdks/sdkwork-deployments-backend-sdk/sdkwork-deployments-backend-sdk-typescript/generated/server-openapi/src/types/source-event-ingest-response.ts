export interface SourceEventIngestResponse {
  eventId: string;
  eventStatus: string;
  buildsTriggered: number;
  duplicate: boolean;
}

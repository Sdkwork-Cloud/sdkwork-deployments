import type {
  ApplicationPublishErrorCode,
  ApplicationPublishStage,
} from './types';

export class ApplicationPublishError extends Error {
  readonly code: ApplicationPublishErrorCode;
  readonly stage: ApplicationPublishStage;

  constructor(
    code: ApplicationPublishErrorCode,
    stage: ApplicationPublishStage,
    message: string,
    cause?: unknown,
  ) {
    super(message, cause === undefined ? undefined : { cause });
    this.name = 'ApplicationPublishError';
    this.code = code;
    this.stage = stage;
  }
}

export function toApplicationPublishError(
  cause: unknown,
  stage: ApplicationPublishStage,
  aborted: boolean,
): ApplicationPublishError {
  if (cause instanceof ApplicationPublishError) {
    return cause;
  }
  if (aborted) {
    return new ApplicationPublishError(
      'ABORTED',
      stage,
      'Application publishing was cancelled.',
      cause,
    );
  }
  return new ApplicationPublishError(
    'STAGE_FAILED',
    stage,
    `Application publishing failed during ${stage}.`,
    cause,
  );
}

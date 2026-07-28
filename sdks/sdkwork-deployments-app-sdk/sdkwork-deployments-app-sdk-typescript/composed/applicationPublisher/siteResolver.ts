import type { SiteResponse } from '../../generated/server-openapi/src/types';
import { ApplicationPublishError } from './errors';
import type {
  ApplicationPublishSiteEvidence,
  ApplicationPublisherDeployClient,
  ResolveOrCreateApplicationPublishSite,
} from './types';

const SITE_LOOKUP_PAGE_SIZE = 50;

interface ExactSiteMatch {
  resolution: 'existingBySlug' | 'existingByName';
  value: SiteResponse;
}

export async function retrieveApplicationPublishSite(
  client: ApplicationPublisherDeployClient,
  siteId: string,
  signal?: AbortSignal,
): Promise<ApplicationPublishSiteEvidence> {
  const value = await client.site.retrieve(siteId, { signal, timeout: undefined });
  const responseId = requireSiteId(value, 'resolveSite');
  if (responseId !== siteId) {
    throw new ApplicationPublishError(
      'SITE_ID_MISMATCH',
      'resolveSite',
      `Resolved Site id ${responseId} does not match requested Site id ${siteId}.`,
    );
  }
  return { id: responseId, resolution: 'existingById', value };
}

export async function findExactApplicationPublishSite(
  client: ApplicationPublisherDeployClient,
  site: ResolveOrCreateApplicationPublishSite,
  signal?: AbortSignal,
): Promise<ApplicationPublishSiteEvidence | undefined> {
  const slug = normalizedOptionalText(site.slug);
  if (slug) {
    const slugResult = await findExactMatch(client, slug, 'slug', signal);
    if (slugResult) {
      return evidenceFromMatch(slugResult);
    }
  }

  const name = site.name.trim();
  const nameResult = await findExactMatch(client, name, 'name', signal);
  if (nameResult) {
    return evidenceFromMatch(nameResult);
  }
  return undefined;
}

export function createdApplicationPublishSiteEvidence(
  value: SiteResponse,
): ApplicationPublishSiteEvidence {
  return {
    id: requireSiteId(value, 'createSite'),
    resolution: 'created',
    value,
  };
}

function evidenceFromMatch(match: ExactSiteMatch): ApplicationPublishSiteEvidence {
  return {
    id: requireSiteId(match.value, 'resolveSite'),
    resolution: match.resolution,
    value: match.value,
  };
}

async function findExactMatch(
  client: ApplicationPublisherDeployClient,
  keyword: string,
  field: 'slug' | 'name',
  signal?: AbortSignal,
): Promise<ExactSiteMatch | undefined> {
  const page = await client.site.list(
    { page: 1, pageSize: SITE_LOOKUP_PAGE_SIZE, keyword },
    { signal, timeout: undefined },
  );
  if (page.pageInfo.hasMore) {
    throw new ApplicationPublishError(
      'SITE_RESOLUTION_AMBIGUOUS',
      'resolveSite',
      `Site lookup for exact ${field} ${keyword} exceeded one bounded result page.`,
    );
  }

  const matches = page.items.filter((candidate) => candidate[field] === keyword);
  if (matches.length > 1) {
    throw new ApplicationPublishError(
      'SITE_RESOLUTION_AMBIGUOUS',
      'resolveSite',
      `Multiple Sites matched exact ${field} ${keyword}.`,
    );
  }
  const value = matches[0];
  return value
    ? {
        resolution: field === 'slug' ? 'existingBySlug' : 'existingByName',
        value,
      }
    : undefined;
}

function requireSiteId(
  value: SiteResponse,
  stage: 'resolveSite' | 'createSite',
): string {
  const id = normalizedOptionalText(value.id);
  if (!id) {
    throw new ApplicationPublishError(
      'SITE_RESPONSE_MISSING_ID',
      stage,
      'Deploy Site response did not include a Site id.',
    );
  }
  return id;
}

function normalizedOptionalText(value: string | undefined): string | undefined {
  const normalized = value?.trim();
  return normalized || undefined;
}

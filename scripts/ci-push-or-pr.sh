#!/usr/bin/env bash
# Push HEAD to the default branch. If the branch is protected (GH006), open a PR.
set -euo pipefail
TITLE="${1:?commit title}"
BODY="${2:-CI could not push to the protected default branch. Review this BUILD_PLAN sync.}"
BRANCH="${DEFAULT_BRANCH:-main}"
if git push origin "HEAD:${BRANCH}"; then
  echo "Pushed to ${BRANCH}"
  exit 0
fi
echo "Push to ${BRANCH} rejected; opening a PR"
RUN="${GITHUB_RUN_ID:-local}"
SHORT="$(git rev-parse --short HEAD)"
HEAD_BRANCH="chore/ci-sync-${RUN}-${SHORT}"
git push -u origin "HEAD:${HEAD_BRANCH}"
if ! command -v gh >/dev/null; then
  echo "gh missing; branch ${HEAD_BRANCH} is on origin" >&2
  exit 0
fi
if gh pr create --base "${BRANCH}" --head "${HEAD_BRANCH}" --title "${TITLE}" --body "${BODY}"; then
  exit 0
fi
EXISTING="$(gh pr list --base "${BRANCH}" --head "${HEAD_BRANCH}" --json number --jq 'length')"
if [ "${EXISTING}" != "0" ]; then
  echo "PR already open for ${HEAD_BRANCH}"
  exit 0
fi
echo "FAIL: could not push ${BRANCH} or open a PR for ${HEAD_BRANCH}" >&2
exit 1

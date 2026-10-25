#!/usr/bin/env python3
"""Expose push/main workflow results as ordinary commit statuses for monitors."""
import json
import os
import sys
from urllib.request import Request, urlopen

context = sys.argv[1]
results = os.environ.get('QUALITY_RESULTS')
state, description = 'pending', 'Workflow is running'
if results is not None:
    jobs = json.loads(results)
    failed = [name for name, job in jobs.items() if job['result'] != 'success']
    state = 'failure' if failed else 'success'
    description = ('Not successful: ' + ', '.join(failed))[:140] if failed else 'All workflow quality gates passed'
url = f"{os.environ['GITHUB_SERVER_URL']}/{os.environ['GITHUB_REPOSITORY']}/actions/runs/{os.environ['GITHUB_RUN_ID']}"
endpoint = f"{os.environ['GITHUB_API_URL']}/repos/{os.environ['GITHUB_REPOSITORY']}/statuses/{os.environ['GITHUB_SHA']}"
request = Request(endpoint, method='POST', data=json.dumps(dict(state=state, context=context, description=description, target_url=url)).encode(), headers={
    'Authorization': 'Bearer ' + os.environ['GITHUB_TOKEN'],
    'Accept': 'application/vnd.github+json', 'Content-Type': 'application/json',
    'X-GitHub-Api-Version': '2022-11-28',
})
with urlopen(request, timeout=30) as response:
    assert response.status == 201
print(f'{context}: {state} — {url}')

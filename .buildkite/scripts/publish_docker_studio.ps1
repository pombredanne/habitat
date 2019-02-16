#!/usr/bin/env powershell

#Requires -Version 5

$ErrorActionPreference="stop" 
$ReleaseChannel = & buildkite-agent meta-data get release-channel

Write-Host "--- Building the Windows Docker Studio"

& $PSScriptRoot/build_docker_image.ps1 -ReleaseChannel=$ReleaseChannel

Write-Host "--- Publishing the Windows Docker Studio"
. ./results/last_image.ps1
Write-Host "Logging in to Bintray Docker repo"
docker login -u="$env:BINTRAY_USER" -p="$env:BINTRAY_KEY" habitat-docker-registry.bintray.io

if((& buildkite-agent meta-data exists fake-release).ExitCode -ne 0) {
  Write-Host "I would publish docker images now. Don't want to try to build until I have this logic and code correct"
  # try {
    # Write-Host "Pushing ${docker_image}:$docker_image_version"
    # docker push "${docker_image}:$docker_image_version"
    # Write-Host "Pushing ${docker_image}:$docker_image_short_version tag for $docker_image_version"
    # docker push "${docker_image}:$docker_image_short_version"
    # Write-Host "Pushing latest tag for $docker_image_version"
    # docker push "${docker_image}:latest"
  # }
  # finally {
      # Remove-Item $HOME/.docker/config.json -Recurse -Force -ErrorAction SilentlyContinue
  # }
} else {
  Write-Host "This is a fake release."
  Write-Host "Not uploading $docker_image"
}

#!/usr/bin/env bun

const commit = "f6e7aefa728585832b6ac737c0fb2bc97461dc16"
const baseUrl = `https://raw.githubusercontent.com/anomalyco/opencode/${commit}`

const typesUrl = `${baseUrl}/packages/sdk/js/src/v2/gen/types.gen.ts`
const apiDocsUrl = `${baseUrl}/packages/sdk/openapi.json`

async function downloadFile(url: string, outputPath: string): Promise<void> {
  const response = await fetch(url)
  if (!response.ok) {
    throw new Error(`Failed to download ${url}: ${response.statusText}`)
  }
  const content = await response.text()
  await Bun.write(outputPath, content)
  console.log(`Downloaded ${outputPath}`)
}

await Promise.all([
  downloadFile(typesUrl, "opencode-types.ts"),
  downloadFile(apiDocsUrl, "opencode-api-docs.json"),
])

console.log("Done!")

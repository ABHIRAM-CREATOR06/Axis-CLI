using PuppeteerSharp;
using System.Diagnostics;
using System.Text.Json;
using System.Text.Json.Serialization;

// Parse arguments
var url = string.Empty;
var timeout = 60000;

for (int i = 0; i < args.Length; i++)
{
    switch (args[i])
    {
        case "--url" when i + 1 < args.Length:
            url = args[++i];
            break;
        case "--json":
            // flag recognised but output is always JSON — kept for CLI compat
            break;
        case "--timeout" when i + 1 < args.Length:
            timeout = int.Parse(args[++i]);
            break;
    }
}

if (string.IsNullOrEmpty(url))
{
    var error = new { error = "No URL provided. Usage: axis-render --url https://example.com --json" };
    Console.WriteLine(JsonSerializer.Serialize(error));
    Environment.Exit(1);
}

// Use a fresh temporary directory for every run so Chrome never serves
// a cached page from a previous check. This is the key fix for identical
// results across different URLs.
// The base dir holds the Chrome binary (stable, downloaded once).
// Each run gets its own uuid subdirectory for its user-data-dir.
var baseDir = Path.Combine(Path.GetTempPath(), "axis-render-chrome");
Directory.CreateDirectory(baseDir);

var runId = Guid.NewGuid().ToString("N")[..8]; // short 8-char id
var userDataDir = Path.Combine(baseDir, "run-" + runId);
Directory.CreateDirectory(userDataDir);

var serializerOptions = new JsonSerializerOptions
{
    PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
    WriteIndented = false,
    DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
};

try
{
    // Only download Chrome when it isn't present yet.
    var fetcherOptions = new BrowserFetcherOptions { Browser = SupportedBrowser.Chrome };
    var browserFetcher = new BrowserFetcher(fetcherOptions);

    var installed = browserFetcher.GetInstalledBrowsers();
    if (!installed.Any())
    {
        Console.Error.WriteLine("  Downloading Chrome (first-time setup)...");
        await browserFetcher.DownloadAsync();
    }

    await using var browser = await Puppeteer.LaunchAsync(new LaunchOptions
    {
        Headless = true,
        Browser = SupportedBrowser.Chrome,
        // Fresh per-run profile — no stale cache across URLs.
        UserDataDir = userDataDir,
        Args = new[]
        {
            "--no-sandbox",
            "--disable-setuid-sandbox",
            "--disable-dev-shm-usage",
            "--disable-gpu",
            "--no-first-run",
            "--no-zygote",
            "--disable-extensions",
            "--disable-background-networking",
            "--disable-default-apps",
            "--disable-sync",
            "--disable-translate",
            "--hide-scrollbars",
            "--metrics-recording-only",
            "--mute-audio",
            "--no-default-browser-check",
            "--safebrowsing-disable-auto-update",
            "--disable-histogram-customizer",
            "--disable-field-trial-config",
            // Disable ALL caching so every run fetches fresh HTML.
            "--disk-cache-size=0",
            "--disable-application-cache",
            "--disable-cache",
        }
    });

    await using var page = await browser.NewPageAsync();

    // Disable cache at the protocol level as well — belt and braces.
    await page.SetCacheEnabledAsync(false);

    var requests = new List<RequestEntry>();
    long totalBytes = 0;

    page.Request += (_, e) =>
    {
        requests.Add(new RequestEntry
        {
            Url = e.Request.Url,
            ResourceType = e.Request.ResourceType.ToString().ToLower(),
            SizeBytes = 0
        });
    };

    page.Response += (_, e) =>
    {
        if (e.Response.Headers.TryGetValue("content-length", out var len)
            && long.TryParse(len, out var bytes))
        {
            totalBytes += bytes;
        }
    };

    await page.SetViewportAsync(new ViewPortOptions { Width = 1280, Height = 800 });

    var stopwatch = Stopwatch.StartNew();

    try
    {
        await page.GoToAsync(url, new NavigationOptions
        {
            WaitUntil = new[] { WaitUntilNavigation.Load },
            Timeout = timeout
        });
    }
    catch (PuppeteerSharp.NavigationException navEx)
    {
        stopwatch.Stop();
        string partialHtml;
        try { partialHtml = await page.GetContentAsync(); }
        catch { partialHtml = string.Empty; }

        if (!string.IsNullOrWhiteSpace(partialHtml))
        {
            Console.Error.WriteLine($"  Navigation timed out but partial HTML captured ({partialHtml.Length} chars).");
            var partial = new RenderResult
            {
                Html = partialHtml,
                Title = string.Empty,
                LoadTimeMs = stopwatch.ElapsedMilliseconds,
                TotalBytes = (long)partialHtml.Length,
                RequestCount = requests.Count,
                Requests = requests,
                Error = null
            };
            Console.WriteLine(JsonSerializer.Serialize(partial, serializerOptions));
            Environment.Exit(0);
        }

        var timeoutResult = new RenderResult
        {
            Html = string.Empty,
            Title = string.Empty,
            LoadTimeMs = stopwatch.ElapsedMilliseconds,
            TotalBytes = 0,
            RequestCount = requests.Count,
            Requests = requests,
            Error = navEx.Message
        };
        Console.WriteLine(JsonSerializer.Serialize(timeoutResult, serializerOptions));
        Environment.Exit(0);
        return;
    }

    stopwatch.Stop();

    var html = await page.GetContentAsync();
    var title = await page.GetTitleAsync();

    byte[]? screenshot = null;
    try
    {
        screenshot = await page.ScreenshotDataAsync(new ScreenshotOptions
        {
            FullPage = true,
            Type = ScreenshotType.Png
        });
    }
    catch { /* screenshot is optional */ }

    var result = new RenderResult
    {
        Html = html,
        Title = title ?? string.Empty,
        LoadTimeMs = stopwatch.ElapsedMilliseconds,
        TotalBytes = totalBytes > 0 ? totalBytes : html.Length,
        RequestCount = requests.Count,
        Requests = requests,
        ScreenshotBase64 = screenshot != null ? Convert.ToBase64String(screenshot) : null,
        Error = null
    };

    Console.WriteLine(JsonSerializer.Serialize(result, serializerOptions));
    Environment.Exit(0);
}
catch (Exception ex)
{
    var error = new RenderResult
    {
        Html = string.Empty,
        Title = string.Empty,
        LoadTimeMs = 0,
        TotalBytes = 0,
        RequestCount = 0,
        Requests = new List<RequestEntry>(),
        Error = ex.Message
    };
    Console.WriteLine(JsonSerializer.Serialize(error, serializerOptions));
    Environment.Exit(0);
}
finally
{
    // Clean up this run's temporary profile directory.
    try { Directory.Delete(userDataDir, recursive: true); } catch { }
}

// ── Data models ───────────────────────────────────────────────────────────────

record RenderResult
{
    [JsonPropertyName("html")]
    public string Html { get; init; } = string.Empty;

    [JsonPropertyName("title")]
    public string Title { get; init; } = string.Empty;

    [JsonPropertyName("load_time_ms")]
    public long LoadTimeMs { get; init; }

    [JsonPropertyName("total_bytes")]
    public long TotalBytes { get; init; }

    [JsonPropertyName("request_count")]
    public int RequestCount { get; init; }

    [JsonPropertyName("requests")]
    public List<RequestEntry> Requests { get; init; } = new();

    [JsonPropertyName("screenshot_base64")]
    public string? ScreenshotBase64 { get; init; }

    [JsonPropertyName("error")]
    public string? Error { get; init; }
}

record RequestEntry
{
    [JsonPropertyName("url")]
    public string Url { get; set; } = string.Empty;

    [JsonPropertyName("resource_type")]
    public string ResourceType { get; set; } = string.Empty;

    [JsonPropertyName("size_bytes")]
    public long SizeBytes { get; set; }
}

using PuppeteerSharp;
using System.Diagnostics;
using System.Text.Json;
using System.Text.Json.Serialization;

// Parse arguments
var url = string.Empty;
var jsonMode = false;
var timeout = 30000;

for (int i = 0; i < args.Length; i++)
{
    switch (args[i])
    {
        case "--url" when i + 1 < args.Length:
            url = args[++i];
            break;
        case "--json":
            jsonMode = true;
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

try
{
    // Download Chrome if not present
    var browserFetcher = new BrowserFetcher();
    await browserFetcher.DownloadAsync();

    await using var browser = await Puppeteer.LaunchAsync(new LaunchOptions
    {
        Headless = true,
        Args = new[]
        {
            "--no-sandbox",
            "--disable-setuid-sandbox",
            "--disable-dev-shm-usage",
            "--disable-gpu",
            "--no-first-run",
            "--no-zygote",
        }
    });

    await using var page = await browser.NewPageAsync();

    // Collect network requests
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

    // Set realistic viewport
    await page.SetViewportAsync(new ViewPortOptions
    {
        Width = 1280,
        Height = 800
    });

    // Navigate and wait for network idle
    var stopwatch = Stopwatch.StartNew();

    await page.GoToAsync(url, new NavigationOptions
    {
        Timeout = timeout
    });

    stopwatch.Stop();

    // Get page content
    var html = await page.GetContentAsync();
    var title = await page.GetTitleAsync();

    // Take screenshot
    byte[]? screenshot = null;
    try
    {
        screenshot = await page.ScreenshotDataAsync(new ScreenshotOptions
        {
            FullPage = true,
            Type = ScreenshotType.Png
        });
    }
    catch
    {
        // Screenshot is optional — don't fail the whole render
    }

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

    var options = new JsonSerializerOptions
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        WriteIndented = false,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
    };

    Console.WriteLine(JsonSerializer.Serialize(result, options));
    Environment.Exit(0);
}
catch (Exception ex)
{
    var error = new
    {
        html = string.Empty,
        title = string.Empty,
        load_time_ms = 0,
        total_bytes = 0,
        request_count = 0,
        requests = Array.Empty<object>(),
        screenshot_base64 = (string?)null,
        error = ex.Message
    };
    Console.WriteLine(JsonSerializer.Serialize(error));
    Environment.Exit(1);
}

// Data models
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

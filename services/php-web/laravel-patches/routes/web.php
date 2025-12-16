<?php

use Illuminate\Support\Facades\Route;

Route::get('/', fn() => redirect('/dashboard'));

// Панели
Route::get('/dashboard', [\App\Http\Controllers\DashboardController::class, 'index']);
Route::get('/osdr',      [\App\Http\Controllers\OsdrController::class,      'index']);

// Прокси к rust_iss
Route::get('/api/iss/last',  [\App\Http\Controllers\ProxyController::class, 'last']);
Route::get('/api/iss/trend', [\App\Http\Controllers\ProxyController::class, 'trend']);

// JWST галерея (JSON)
Route::get('/api/jwst/feed', [\App\Http\Controllers\DashboardController::class, 'jwstFeed']);
Route::get("/api/astro/events", [\App\Http\Controllers\AstroController::class, "events"]);

// Telemetry API
Route::get('/api/telemetry', function() {
    try {
        $items = \Illuminate\Support\Facades\DB::select("
            SELECT id, recorded_at, voltage, temp, source_file
            FROM telemetry_legacy
            ORDER BY recorded_at DESC
            LIMIT 20
        ");
        return response()->json(['items' => $items]);
    } catch (\Throwable $e) {
        return response()->json(['error' => $e->getMessage()], 500);
    }
});

// ISS History API
Route::get('/api/iss/history', function() {
    try {
        $items = \Illuminate\Support\Facades\DB::select("
            SELECT id, fetched_at, payload
            FROM iss_fetch_log
            ORDER BY fetched_at DESC
            LIMIT 10
        ");
        foreach ($items as &$item) {
            $item->payload = json_decode($item->payload, true);
        }
        return response()->json(['items' => $items]);
    } catch (\Throwable $e) {
        return response()->json(['error' => $e->getMessage()], 500);
    }
});

// CMS pages
Route::get('/page/{slug}', [\App\Http\Controllers\CmsController::class, 'page']);

<?php

namespace App\Http\Controllers;

use Illuminate\Support\Facades\DB;

class TelemetryController extends Controller
{
    public function list()
    {
        try {
            $items = DB::select("
                SELECT id, recorded_at, voltage, temp, source_file
                FROM telemetry_legacy
                ORDER BY recorded_at DESC
                LIMIT 20
            ");

            return response()->json(['items' => $items]);
        } catch (\Throwable $e) {
            return response()->json(['error' => $e->getMessage()], 500);
        }
    }
}

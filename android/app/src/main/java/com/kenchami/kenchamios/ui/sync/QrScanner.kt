package com.kenchami.kenchamios.ui.sync

import android.content.Context
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning

/**
 * Scans a QR code with the Google code scanner: it runs inside Google Play services, so this app needs no camera permission.
 * Without Play services [onError] is called and the pairing text can be pasted instead.
 */
fun scanQrCode(context: Context, onResult: (String) -> Unit, onError: () -> Unit) {
    val options = GmsBarcodeScannerOptions.Builder().setBarcodeFormats(Barcode.FORMAT_QR_CODE).build()
    GmsBarcodeScanning.getClient(context, options).startScan()
        .addOnSuccessListener { barcode -> barcode.rawValue?.let(onResult) ?: onError() }
        .addOnFailureListener { onError() }
}

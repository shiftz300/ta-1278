// Bounded image/video decoding with the installed Apple frameworks.
// Only media copied from a mounted VFS volume is accepted. No network URLs.
import Foundation
import AVFoundation
import CoreImage
import ImageIO
import UniformTypeIdentifiers

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data((message + "\n").utf8)); exit(1)
}
guard CommandLine.arguments.count == 3 else { fail("Expected image|video and a local file") }
let url = URL(fileURLWithPath: CommandLine.arguments[2])
if CommandLine.arguments[1] == "image" {
    guard let source = CGImageSourceCreateWithURL(url as CFURL, nil),
          let image = CGImageSourceCreateThumbnailAtIndex(source, 0, [
            kCGImageSourceCreateThumbnailFromImageAlways: true,
            kCGImageSourceThumbnailMaxPixelSize: 960,
            kCGImageSourceCreateThumbnailWithTransform: true
          ] as CFDictionary) else { fail("Cannot decode image") }
    let bytes = NSMutableData()
    guard let dest = CGImageDestinationCreateWithData(bytes, UTType.png.identifier as CFString, 1, nil) else { fail("Cannot encode preview") }
    CGImageDestinationAddImage(dest, image, nil)
    guard CGImageDestinationFinalize(dest) else { fail("Cannot finish preview") }
    FileHandle.standardOutput.write(bytes as Data)
    exit(0)
}
guard CommandLine.arguments[1] == "video" else { fail("Unknown operation") }
let item = AVPlayerItem(url: url)
let output = AVPlayerItemVideoOutput(pixelBufferAttributes: [kCVPixelBufferPixelFormatTypeKey as String:kCVPixelFormatType_32BGRA])
item.add(output)
let player = AVPlayer(playerItem: item)
let context = CIContext(options: [.cacheIntermediates:false])
var paused = false
var lastFrame = Data(repeating:0, count:240*240*4)
var finished = false
NotificationCenter.default.addObserver(forName: .AVPlayerItemDidPlayToEndTime, object: item, queue: .main) { _ in finished = true }
DispatchQueue.global().async {
    while let line = readLine() {
        DispatchQueue.main.async {
            let parts = line.split(separator:" ")
            switch parts.first {
            case "pause": paused = true; player.pause()
            case "play":
                if finished { player.seek(to:.zero); finished = false }
                paused = false; player.play()
            case "seek":
                if parts.count == 2, let n = Double(parts[1]), n.isFinite {
                    player.seek(to:CMTime(seconds:max(0,n), preferredTimescale:600), toleranceBefore:.zero, toleranceAfter:.zero); finished = false
                }
            case "volume":
                if parts.count == 2, let n = Float(parts[1]) { player.volume = min(1,max(0,n/10)) }
            case "quit": exit(0)
            default: break
            }
        }
    }
    exit(0)
}
player.volume = 0.6
player.play()
let timer = Timer.scheduledTimer(withTimeInterval:0.1, repeats:true) { _ in
    if item.status == .failed { fail(item.error?.localizedDescription ?? "Video unavailable") }
    let time = player.currentTime()
    if let buffer = output.copyPixelBuffer(forItemTime:time, itemTimeForDisplay:nil),
       let image = context.createCGImage(CIImage(cvPixelBuffer:buffer), from:CIImage(cvPixelBuffer:buffer).extent) {
        var data = Data(repeating:0, count:240*240*4)
        data.withUnsafeMutableBytes { raw in
            if let cg = CGContext(data:raw.baseAddress, width:240, height:240, bitsPerComponent:8, bytesPerRow:960, space:CGColorSpaceCreateDeviceRGB(), bitmapInfo:CGImageAlphaInfo.premultipliedLast.rawValue | CGBitmapInfo.byteOrder32Big.rawValue) {
                cg.setFillColor(CGColor(gray:0,alpha:1)); cg.fill(CGRect(x:0,y:0,width:240,height:240))
                let scale = min(240/Double(image.width),240/Double(image.height))
                let w = Double(image.width)*scale, h = Double(image.height)*scale
                cg.draw(image, in:CGRect(x:(240-w)/2,y:(240-h)/2,width:w,height:h))
            }
        }
        lastFrame = data
    }
    var packet = Data()
    var seconds = time.seconds.isFinite ? time.seconds : 0
    var duration = item.duration.seconds.isFinite ? item.duration.seconds : 0
    withUnsafeBytes(of:&seconds) { packet.append(contentsOf:$0) }
    withUnsafeBytes(of:&duration) { packet.append(contentsOf:$0) }
    packet.append((paused || finished) ? 0 : 1)
    packet.append(lastFrame)
    FileHandle.standardOutput.write(packet)
}
RunLoop.main.run()

// Minimal HID output/feature writer for ChromaFlow Windows lighting (Arena / Prime).
// Compiled at runtime with .NET Framework csc into %LOCALAPPDATA%\chromaflow\bin\CfHidW.exe
using System;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

public class CfHidW
{
    [DllImport("hid.dll", SetLastError = true)]
    static extern bool HidD_GetHidGuid(out Guid g);

    [DllImport("hid.dll", SetLastError = true)]
    static extern bool HidD_GetPreparsedData(SafeFileHandle h, out IntPtr p);

    [DllImport("hid.dll", SetLastError = true)]
    static extern bool HidD_FreePreparsedData(IntPtr p);

    [DllImport("hid.dll", SetLastError = true)]
    static extern uint HidP_GetCaps(IntPtr p, ref HIDP_CAPS c);

    [DllImport("setupapi.dll", SetLastError = true)]
    static extern IntPtr SetupDiGetClassDevs(ref Guid g, IntPtr e, IntPtr p, uint f);

    [DllImport("setupapi.dll", SetLastError = true)]
    static extern bool SetupDiEnumDeviceInterfaces(
        IntPtr h, IntPtr i, ref Guid g, uint idx, ref SP_DEVICE_INTERFACE_DATA d);

    [DllImport("setupapi.dll", SetLastError = true, CharSet = CharSet.Auto)]
    static extern bool SetupDiGetDeviceInterfaceDetail(
        IntPtr h, ref SP_DEVICE_INTERFACE_DATA d, IntPtr det, uint sz, out uint req, IntPtr info);

    [DllImport("setupapi.dll", SetLastError = true)]
    static extern bool SetupDiDestroyDeviceInfoList(IntPtr h);

    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    static extern SafeFileHandle CreateFile(
        string n, uint a, uint s, IntPtr p, uint d, uint f, IntPtr t);

    [DllImport("hid.dll", SetLastError = true)]
    static extern bool HidD_SetOutputReport(SafeFileHandle h, byte[] b, int l);

    [DllImport("hid.dll", SetLastError = true)]
    static extern bool HidD_SetFeature(SafeFileHandle h, byte[] b, int l);

    [StructLayout(LayoutKind.Sequential)]
    struct SP_DEVICE_INTERFACE_DATA
    {
        public int cbSize;
        public Guid InterfaceClassGuid;
        public int Flags;
        public IntPtr Reserved;
    }

    [StructLayout(LayoutKind.Sequential)]
    struct HIDP_CAPS
    {
        public short Usage;
        public short UsagePage;
        public short InputReportByteLength;
        public short OutputReportByteLength;
        public short FeatureReportByteLength;
        public short R0, R1, R2, R3, R4, R5, R6, R7, R8, R9, R10, R11, R12, R13, R14, R15, R16;
        public short NumberLinkCollectionNodes;
        public short NumberInputButtonCaps;
        public short NumberInputValueCaps;
        public short NumberInputDataIndices;
        public short NumberOutputButtonCaps;
        public short NumberOutputValueCaps;
        public short NumberOutputDataIndices;
        public short NumberFeatureButtonCaps;
        public short NumberFeatureValueCaps;
        public short NumberFeatureDataIndices;
    }

    const uint DIGCF_PRESENT = 2, DIGCF_DEVICEINTERFACE = 0x10;
    const uint GENERIC_READ = 0x80000000, GENERIC_WRITE = 0x40000000;
    const uint FILE_SHARE_READ = 1, FILE_SHARE_WRITE = 2, OPEN_EXISTING = 3;

    static byte[] Pad(byte[] src, int len)
    {
        if (len <= 0 || src.Length == len)
            return src;
        byte[] dst = new byte[len];
        Buffer.BlockCopy(src, 0, dst, 0, Math.Min(src.Length, len));
        return dst;
    }

    public static int Main(string[] args)
    {
        if (args.Length < 3)
        {
            Console.Error.WriteLine("usage: CfHidW VID PID HEXBYTES");
            return 2;
        }
        string want = ("VID_" + args[0] + "&PID_" + args[1]).ToUpperInvariant();
        byte[] report = new byte[args[2].Length / 2];
        for (int i = 0; i < report.Length; i++)
            report[i] = Convert.ToByte(args[2].Substring(i * 2, 2), 16);

        Guid guid;
        HidD_GetHidGuid(out guid);
        IntPtr h = SetupDiGetClassDevs(
            ref guid, IntPtr.Zero, IntPtr.Zero, DIGCF_PRESENT | DIGCF_DEVICEINTERFACE);
        try
        {
            // x64 Unicode SP_DEVICE_INTERFACE_DETAIL_DATA.cbSize must be 8 (not 4).
            int detailCb = IntPtr.Size == 8 ? 8 : 6;
            for (uint i = 0; ; i++)
            {
                SP_DEVICE_INTERFACE_DATA did = new SP_DEVICE_INTERFACE_DATA();
                did.cbSize = Marshal.SizeOf(did);
                if (!SetupDiEnumDeviceInterfaces(h, IntPtr.Zero, ref guid, i, ref did))
                    break;
                uint req;
                SetupDiGetDeviceInterfaceDetail(h, ref did, IntPtr.Zero, 0, out req, IntPtr.Zero);
                IntPtr det = Marshal.AllocHGlobal((int)req);
                try
                {
                    Marshal.WriteInt32(det, detailCb);
                    if (!SetupDiGetDeviceInterfaceDetail(h, ref did, det, req, out req, IntPtr.Zero))
                        continue;
                    string path = Marshal.PtrToStringAuto(IntPtr.Add(det, 4));
                    if (string.IsNullOrEmpty(path))
                        continue;
                    if (!path.StartsWith("\\\\"))
                        path = "\\\\" + path.TrimStart('\\');
                    if (path.ToUpperInvariant().IndexOf(want) < 0)
                        continue;
                    if (path.EndsWith("\\kbd", StringComparison.OrdinalIgnoreCase))
                        continue;
                    var fh = CreateFile(
                        path,
                        GENERIC_READ | GENERIC_WRITE,
                        FILE_SHARE_READ | FILE_SHARE_WRITE,
                        IntPtr.Zero,
                        OPEN_EXISTING,
                        0,
                        IntPtr.Zero);
                    if (fh.IsInvalid)
                        continue;

                    IntPtr prep;
                    short outLen = 0, featLen = 0;
                    if (HidD_GetPreparsedData(fh, out prep))
                    {
                        HIDP_CAPS caps = new HIDP_CAPS();
                        HidP_GetCaps(prep, ref caps);
                        outLen = caps.OutputReportByteLength;
                        featLen = caps.FeatureReportByteLength;
                        HidD_FreePreparsedData(prep);
                    }

                    bool ok = false;
                    if (outLen > 0)
                        ok = HidD_SetOutputReport(fh, Pad(report, outLen), outLen);
                    if (!ok && featLen > 0)
                        ok = HidD_SetFeature(fh, Pad(report, featLen), featLen);
                    if (!ok)
                        ok = HidD_SetOutputReport(fh, report, report.Length)
                            || HidD_SetFeature(fh, report, report.Length);
                    fh.Dispose();
                    if (ok)
                    {
                        Console.WriteLine("ok " + path);
                        return 0;
                    }
                }
                finally
                {
                    Marshal.FreeHGlobal(det);
                }
            }
        }
        finally
        {
            SetupDiDestroyDeviceInfoList(h);
        }
        Console.Error.WriteLine("no writable HID " + want);
        return 1;
    }
}

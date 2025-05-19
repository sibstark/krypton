// app/pay/[id]/page.tsx (или pages/pay/[id].tsx, если без appDir)

'use client';

import { useEffect, useState } from 'react';
import { TonConnectUIProvider, TonConnectButton, useTonConnectUI } from '@tonconnect/ui-react';
import { useParams } from 'next/navigation';

export default function PayPage() {
    const params = useParams();
    const id = params?.id as string;

    const [qrUrl, setQrUrl] = useState<string | null>(null);
    const [tonLink, setTonLink] = useState<string | null>(null);
    const [status, setStatus] = useState<'idle' | 'loading' | 'ready'>('idle');

    // const [tonConnectUI] = useTonConnectUI();

    useEffect(() => {
        const fetchQR = async () => {
            setStatus('loading');
            try {
                const res = await fetch(`/api/payment/${id}/start`, {
                    method: 'POST',
                });
                const data = await res.json();
                setQrUrl(data.qr_base64);
                setTonLink(data.ton_link);
                setStatus('ready');
            } catch (err) {
                console.error('Failed to fetch QR:', err);
                setStatus('idle');
            }
        };
        if (id) { fetchQR() };
    }, [id]);

    return (
        <TonConnectUIProvider manifestUrl="/tonconnect-manifest.json">
            <div className="min-h-screen flex flex-col items-center justify-center p-4">
                <h1 className="text-2xl font-bold mb-4">Pay with TON</h1>
                <TonConnectButton />

                {status === 'loading' && <p className="mt-6">Loading payment...</p>}

                {status === 'ready' && qrUrl && (
                    <>
                        <img src={qrUrl} alt="QR Code" className="mt-6 w-64 h-64" />
                        <a href={tonLink!} className="mt-4 text-blue-600 underline" target="_blank">
                            Open in wallet
                        </a>
                    </>
                )}

                {status === 'idle' && <p className="mt-6 text-red-600">Failed to load QR</p>}
            </div>
        </TonConnectUIProvider>
    );
}
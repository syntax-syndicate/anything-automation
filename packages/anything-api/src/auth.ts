import { SupabaseClient } from '@supabase/supabase-js';

const ANYTHING_API_URL = process.env.NEXT_PUBLIC_ANYTHING_API_URL

export const getProvider = async (supabase: SupabaseClient, account_id: string, provider_name: string) => {
    try {
        console.log('getting provider_name in anything_api/auth:', provider_name);
        const { data: { session } } = await supabase.auth.getSession();
 
        console.log('Session:', session);

        if (session) {
            const response = await fetch(`${ANYTHING_API_URL}/account/${account_id}/auth/providers/${provider_name}`, {
                headers: {
                    Authorization: `${session.access_token}`,
                },
            });
            const data = await response.json();
            console.log('Data from /api/auth/:provider_name', data);
            return data;
        }
    } catch (error) {
        console.error('Error fetching provider by provider_name:', error);
    } 
}

export const getAuthAccounts = async (supabase: SupabaseClient, account_id: string) => {
    try {
        const { data: { session } } = await supabase.auth.getSession();

        console.log('Session:', session);

        if (session) {
            const response = await fetch(`${ANYTHING_API_URL}/account/${account_id}/auth/accounts`, {
                headers: {
                    Authorization: `${session.access_token}`,
                },
            });
            const data = await response.json();
            console.log('Data from /api/auth/accounts', data);
            return data;
        }
    } catch (error) {
        console.error('Error fetching auth accounts', error);
    } 
}

export const getAuthAccountsForProvider = async (supabase: SupabaseClient, account_id: string, provider_name: string) => {
    try {
        console.log('getting auth accounts for provider: ', provider_name);
        const { data: { session } } = await supabase.auth.getSession();

        console.log('Session:', session);

        if (session) {
            const response = await fetch(`${ANYTHING_API_URL}/account/${account_id}/auth/accounts/${provider_name}`, {
                headers: {
                    Authorization: `${session.access_token}`,
                },
            });
            const data = await response.json();
            console.log('Data from /api/auth/accounts/:provider_name', data);
            return data;
        }

    } catch (error) {
        console.error('Error fetching auth accounts for provider:', error);
    } 
}

export const getProviders = async (supabase: SupabaseClient, account_id: string) => {
    try {
        const { data: { session } } = await supabase.auth.getSession();

        console.log('Session:', session);

        if (session) {
            const response = await fetch(`${ANYTHING_API_URL}/account/${account_id}/auth/providers`, {
                headers: {
                    Authorization: `${session.access_token}`,
                },
            });
            const data = await response.json();
            console.log('Data from /api/auth/providers', data);
            return data;
        }
    } catch (error) {
        console.error('Error fetching auth providers', error);
    } 
}

export const handleCallbackForProvider = async (supabase: SupabaseClient, {account_id, provider_name, code, state}: {account_id: string, provider_name: string, code: any, state: any}) => {
    try {
        console.log('handling callback for provider: ', provider_name);
        const userData = await supabase.auth.getUser(); 
        console.log('User Data:', userData);

        const { data: { session }, error } = await supabase.auth.getSession();

        console.log('Session:', session);
        console.log('Error:', error);

        if (session) {
            console.log("calling /api/auth/:provider_name/callback");
            const response = await fetch(`${ANYTHING_API_URL}/account/${account_id}/auth/${provider_name}/callback`, {
                method: 'POST',
                headers: {
                  'Content-Type': 'application/json',
                  Authorization: `${session.access_token}`,
                },
                body: JSON.stringify({
                  code,
                  state
                }),
            });
            const data = await response.json();
            console.log('Data from /api/auth/:provider_name/callback', data);
            return data;
        } else {
            console.error('No session found in handleCallbackForProvider');
        }
    } catch (error) {
        console.error('Error handling callback', error);
    } 
}

export const initiateProviderAuth = async (supabase: SupabaseClient, account_id: string, provider_name: string) => {
    try {
        const { data: { session } } = await supabase.auth.getSession();

        console.log('Session:', session);

        if (session) {
            const response = await fetch(`${ANYTHING_API_URL}/account/${account_id}/auth/${provider_name}/initiate`, {
                headers: {
                    Authorization: `${session.access_token}`,
                },
            });
            const data = await response.json();
            console.log('Data from /api/auth/:provider_name/initiate', data);
            return data;
        }
    } catch (error) {
        console.error('Error initiating provider auth', error);
    } 
}

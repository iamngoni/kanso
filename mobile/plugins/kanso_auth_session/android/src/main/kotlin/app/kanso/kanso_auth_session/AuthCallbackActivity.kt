package app.kanso.kanso_auth_session

import android.app.Activity
import android.os.Bundle

/**
 * Where `kanso://auth/...` lands. It gives the URL to the open session,
 * brings the app in front (which closes the Custom Tab) and finishes.
 */
class AuthCallbackActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (AuthSessions.deliver(intent?.data)) {
            AuthSessions.bringAppToFront(this)
        }
        finish()
    }
}

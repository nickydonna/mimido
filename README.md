# MimiDo

This project is a personal organizer, using CalDAV as a backend. The main objective is to organize myself in a better way.

My main issue with current task manager is that everything is an event or a task, which it annoying to organize around, so the approach used is to provide more tools, the ones provided now are:

* Block: A block of time dedicated to something. For example, work, dinner, family time, etc. This will allow for quickly viewing what you are doing in that time. For now is not blocking, but it can change in the future for free/busy queries.
* Event: A typical event, usually time blocking and fixed in time. A future version will include "transport time" and "preparation time". Example: I need to go to the doctor at 13 pm, it takes me 15 min to go, and I need 30 min to take a bath or look for previous studies to share.
* Task: Something that needs to be done, can have time or not. Example: Fix Bug 15, Go grocery shopping
* Reminder: An small thing that you not forget, should not take more that 15 min. Some of this Reminder may be "Create X Task". Example: Take the chicken out of the freezer, make doctors appoiment

# Backend

We are using CalDAV as a backend. CalDAV is a common standard for Calendar used by most platforms, Google, Apple, Fastmail, etc.

Using this we can leverage calendar notification from cellphones, no scaling needed, and the user owns the data.

To access it there are different mechanisms, but for now we support only Basic Auth.

## Authentication

Since we don't have any backend, right now the credentials are store in a secure cookie in a jwt format, so only use this on trusted devices. This will eventually help with creating a PWA.

A minor upgrade would be encrypting the cookie.

## Note on Fastmail

Fastmail support both VEVENT and VTODO but because of a weird functionality on iOS they split it into two collections.

To make a single calendar support both you need to run the following command:

```sh
curl --location --request PROPPATCH '<url of your calendar>' --header 'Content-Type: application/xml' --header 'Authorization: Basic <your auth>' --data '<?xml version="1.0" encoding="utf-8" ?>
<propertyupdate xmlns="DAV:">
	<set>
		<prop>
			<C:supported-calendar-component-set
				xmlns:C="urn:ietf:params:xml:ns:caldav" force="yes">
				<C:comp name="VEVENT"/>
				<C:comp name="VTODO"/>
			</C:supported-calendar-component-set>
		</prop>
	</set>
</propertyupdate>'
```

Eventually we will an option to enable this in the app
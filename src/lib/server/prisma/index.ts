
import { PrismaClient } from '@prisma/client'
import bcrypt from 'bcrypt'

export const prisma = new PrismaClient().$extends({
  query: {
    user: {
      $allOperations({ operation, args, query }) {
        // @ts-expect-error subtypings
        if (['create', 'update'].includes(operation) && args.data && args.data['password']) {
          // @ts-expect-error subtypings
          args.data['password'] = bcrypt.hashSync(args.data['password'], 10)
        }
        return query(args)
      }
    }
  }
});


